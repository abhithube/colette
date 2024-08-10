use colette_core::{
    common::FindOneParams,
    feeds::{
        Error, FeedsCreateData, FeedsFindManyParams, FeedsRepository, FeedsUpdateData, StreamFeed,
    },
};
use colette_entities::{
    entry, feed, feed_entry, profile_feed, profile_feed_entry, profile_feed_tag, tag,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    prelude::Expr,
    sea_query::{Func, OnConflict, Query},
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, LoaderTrait, ModelTrait,
    QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{tags::Tag, PostgresRepository};

#[async_trait::async_trait]
impl FeedsRepository for PostgresRepository {
    async fn find_many_feeds(
        &self,
        params: FeedsFindManyParams,
    ) -> Result<Vec<colette_core::Feed>, Error> {
        sqlx::query_file_as!(
            Feed,
            "queries/feeds/find_many.sql",
            params.profile_id,
            params.tags.as_deref()
        )
        .fetch_all(self.db.get_postgres_connection_pool())
        .await
        .map(|e| e.into_iter().map(colette_core::Feed::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_feed(&self, params: FindOneParams) -> Result<colette_core::Feed, Error> {
        sqlx::query_file_as!(
            Feed,
            "queries/feeds/find_one.sql",
            params.id,
            params.profile_id
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Feed::from)
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn create_feed(&self, data: FeedsCreateData) -> Result<colette_core::Feed, Error> {
        let id = self
            .db
            .transaction::<_, Uuid, Error>(|txn| {
                Box::pin(async move {
                    let link = data.feed.link.to_string();
                    let active_model = feed::ActiveModel {
                        link: Set(link.clone()),
                        title: Set(data.feed.title),
                        url: Set(if data.url == link {
                            None
                        } else {
                            Some(data.url)
                        }),
                        ..Default::default()
                    };

                    let result = feed::Entity::insert(active_model)
                        .on_conflict(
                            OnConflict::column(feed::Column::Link)
                                .update_columns([feed::Column::Title, feed::Column::Url])
                                .to_owned(),
                        )
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                    let feed_id = result.last_insert_id;

                    let active_model = profile_feed::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        profile_id: Set(data.profile_id),
                        feed_id: Set(feed_id),
                        ..Default::default()
                    };

                    let pf_id = match profile_feed::Entity::insert(active_model)
                        .on_conflict(
                            OnConflict::columns([
                                profile_feed::Column::ProfileId,
                                profile_feed::Column::FeedId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(txn)
                        .await
                    {
                        Ok(result) => Ok(result.last_insert_id),
                        Err(DbErr::RecordNotInserted) => {
                            let Some(model) = profile_feed::Entity::find()
                                .filter(profile_feed::Column::ProfileId.eq(data.profile_id))
                                .filter(profile_feed::Column::FeedId.eq(feed_id))
                                .one(txn)
                                .await
                                .map_err(|e| Error::Unknown(e.into()))?
                            else {
                                return Err(Error::Unknown(anyhow::anyhow!(
                                    "Failed to fetch created profile feed"
                                )));
                            };

                            Ok(model.id)
                        }
                        Err(e) => Err(Error::Unknown(e.into())),
                    }?;

                    let links = data
                        .feed
                        .entries
                        .iter()
                        .map(|e| e.link.to_string())
                        .collect::<Vec<_>>();

                    let active_models = data
                        .feed
                        .entries
                        .into_iter()
                        .map(|e| entry::ActiveModel {
                            link: Set(e.link.to_string()),
                            title: Set(e.title),
                            published_at: Set(e.published.map(|e| e.into())),
                            description: Set(e.description),
                            author: Set(e.author),
                            thumbnail_url: Set(e.thumbnail.map(String::from)),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();

                    entry::Entity::insert_many(active_models)
                        .on_empty_do_nothing()
                        .on_conflict(
                            OnConflict::column(entry::Column::Link)
                                .update_columns([
                                    entry::Column::Title,
                                    entry::Column::PublishedAt,
                                    entry::Column::Description,
                                    entry::Column::Author,
                                    entry::Column::ThumbnailUrl,
                                ])
                                .to_owned(),
                        )
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let entry_models = entry::Entity::find()
                        .filter(entry::Column::Link.is_in(links))
                        .all(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;
                    let entry_ids = entry_models.iter().map(|e| e.id).collect::<Vec<_>>();

                    let active_models = entry_models
                        .into_iter()
                        .map(|e| feed_entry::ActiveModel {
                            feed_id: Set(feed_id),
                            entry_id: Set(e.id),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();

                    feed_entry::Entity::insert_many(active_models)
                        .on_empty_do_nothing()
                        .on_conflict(
                            OnConflict::columns([
                                feed_entry::Column::FeedId,
                                feed_entry::Column::EntryId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let fe_models = feed_entry::Entity::find()
                        .filter(feed_entry::Column::FeedId.eq(feed_id))
                        .filter(feed_entry::Column::EntryId.is_in(entry_ids))
                        .all(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let active_models = fe_models
                        .into_iter()
                        .map(|e| profile_feed_entry::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            profile_feed_id: Set(pf_id),
                            feed_entry_id: Set(e.id),
                            profile_id: Set(data.profile_id),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();

                    profile_feed_entry::Entity::insert_many(active_models)
                        .on_empty_do_nothing()
                        .on_conflict(
                            OnConflict::columns([
                                profile_feed_entry::Column::ProfileFeedId,
                                profile_feed_entry::Column::FeedEntryId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(pf_id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => {
                    println!("{:?}", e);
                    e
                }
                _ => Error::Unknown(e.into()),
            })?;

        self.find_one_feed(FindOneParams {
            id,
            profile_id: data.profile_id,
        })
        .await
    }

    async fn update_feed(
        &self,
        params: FindOneParams,
        data: FeedsUpdateData,
    ) -> Result<colette_core::Feed, Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(pf_model) = profile_feed::Entity::find_by_id(params.id)
                        .filter(profile_feed::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    let mut active_model = pf_model.clone().into_active_model();
                    if let Some(title) = data.title {
                        active_model.title = Set(title)
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    if let Some(tags) = data.tags {
                        let active_models = tags
                            .clone()
                            .into_iter()
                            .map(|title| tag::ActiveModel {
                                id: Set(Uuid::new_v4()),
                                title: Set(title.clone()),
                                slug: Set(slug::slugify(title)),
                                profile_id: Set(params.profile_id),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();

                        tag::Entity::insert_many(active_models)
                            .on_empty_do_nothing()
                            .on_conflict(
                                OnConflict::columns([tag::Column::ProfileId, tag::Column::Title])
                                    .do_nothing()
                                    .to_owned(),
                            )
                            .exec(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let tag_models = tag::Entity::find()
                            .filter(tag::Column::Title.is_in(&tags))
                            .all(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                        let tag_ids = tag_models.iter().map(|e| e.id).collect::<Vec<_>>();

                        profile_feed_tag::Entity::delete_many()
                            .filter(profile_feed_tag::Column::TagId.is_not_in(tag_ids.clone()))
                            .exec(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let active_models = tag_ids
                            .into_iter()
                            .map(|tag_id| profile_feed_tag::ActiveModel {
                                profile_feed_id: Set(pf_model.id),
                                tag_id: Set(tag_id),
                                profile_id: Set(params.profile_id),
                                ..Default::default()
                            })
                            .collect::<Vec<_>>();

                        profile_feed_tag::Entity::insert_many(active_models)
                            .on_empty_do_nothing()
                            .on_conflict(
                                OnConflict::columns([
                                    profile_feed_tag::Column::ProfileFeedId,
                                    profile_feed_tag::Column::TagId,
                                ])
                                .do_nothing()
                                .to_owned(),
                            )
                            .exec(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        self.find_one_feed(params).await
    }

    async fn delete_feed(&self, params: FindOneParams) -> Result<(), Error> {
        let result = profile_feed::Entity::delete_by_id(params.id)
            .filter(profile_feed::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }

    async fn stream_feeds(&self) -> Result<BoxStream<Result<StreamFeed, Error>>, Error> {
        feed::Entity::find()
            .expr_as(
                Func::coalesce([
                    Expr::col(feed::Column::Url).into(),
                    Expr::col(feed::Column::Link).into(),
                ]),
                "url",
            )
            .into_model::<StreamSelect>()
            .stream(&self.db)
            .await
            .map(|e| {
                e.map(|e| {
                    e.map(StreamFeed::from)
                        .map_err(|e| Error::Unknown(e.into()))
                })
                .map_err(|e| Error::Unknown(e.into()))
                .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_feeds(&self) -> Result<(), Error> {
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let subquery = Query::select()
                        .from(profile_feed_entry::Entity)
                        .and_where(
                            Expr::col((
                                profile_feed_entry::Entity,
                                profile_feed_entry::Column::FeedEntryId,
                            ))
                            .equals((feed_entry::Entity, feed_entry::Column::Id)),
                        )
                        .to_owned();

                    let result = feed_entry::Entity::delete_many()
                        .filter(Expr::exists(subquery).not())
                        .exec(txn)
                        .await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feed entries", result.rows_affected);
                    }

                    let subquery = Query::select()
                        .from(feed_entry::Entity)
                        .and_where(
                            Expr::col((feed_entry::Entity, feed_entry::Column::EntryId))
                                .equals((entry::Entity, entry::Column::Id)),
                        )
                        .to_owned();

                    let result = entry::Entity::delete_many()
                        .filter(Expr::exists(subquery).not())
                        .exec(txn)
                        .await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned entries", result.rows_affected);
                    }

                    let subquery = Query::select()
                        .from(profile_feed::Entity)
                        .and_where(
                            Expr::col((profile_feed::Entity, profile_feed::Column::FeedId))
                                .equals((feed::Entity, feed::Column::Id)),
                        )
                        .to_owned();

                    let result = feed::Entity::delete_many()
                        .filter(Expr::exists(subquery).not())
                        .exec(txn)
                        .await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feeds", result.rows_affected);
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Clone, Debug)]
struct Feed {
    id: Uuid,
    link: String,
    title: Option<String>,
    original_title: String,
    url: Option<String>,
    tags: Option<Json<Vec<Tag>>>,
    unread_count: Option<i64>,
}

impl From<Feed> for colette_core::Feed {
    fn from(value: Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            original_title: value.original_title,
            url: value.url,
            tags: value
                .tags
                .map(|e| e.0.into_iter().map(colette_core::Tag::from).collect()),
            unread_count: value.unread_count,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct StreamSelect {
    pub id: i32,
    pub url: String,
}

impl From<StreamSelect> for StreamFeed {
    fn from(value: StreamSelect) -> Self {
        Self {
            id: value.id,
            url: value.url,
        }
    }
}
