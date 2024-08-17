use std::collections::HashMap;

use anyhow::anyhow;
use colette_core::{
    common::Paginated,
    feed::{
        Error, FeedCreateData, FeedFindManyFilters, FeedRepository, FeedUpdateData, StreamFeed,
    },
    Feed,
};
use colette_entities::{
    feed, feed_entry, profile_feed, profile_feed_entry, profile_feed_tag, tag,
    PfWithFeedAndTagsAndUnreadCount,
};
use colette_utils::base_64;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    prelude::Expr,
    sea_query::{Func, OnConflict, Query, SimpleExpr},
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
    JoinType, LoaderTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::SqlRepository;

#[async_trait::async_trait]
impl FeedRepository for SqlRepository {
    async fn find_many_feeds(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Paginated<Feed>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn find_one_feed(&self, id: Uuid, profile_id: Uuid) -> Result<Feed, Error> {
        find_by_id(&self.db, id, profile_id).await
    }

    async fn create_feed(&self, data: FeedCreateData) -> Result<Feed, Error> {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
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

                    let mut active_model = profile_feed::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        profile_id: Set(data.profile_id),
                        feed_id: Set(feed_id),
                        ..Default::default()
                    };
                    if let Some(folder_id) = data.folder_id {
                        active_model.folder_id = Set(folder_id);
                    }

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
                                return Err(Error::Unknown(anyhow!(
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
                        .map(|e| feed_entry::ActiveModel {
                            link: Set(e.link.to_string()),
                            title: Set(e.title),
                            published_at: Set(e.published.map(|e| e.into())),
                            description: Set(e.description),
                            author: Set(e.author),
                            thumbnail_url: Set(e.thumbnail.map(String::from)),
                            feed_id: Set(feed_id),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();

                    feed_entry::Entity::insert_many(active_models)
                        .on_empty_do_nothing()
                        .on_conflict(
                            OnConflict::columns([
                                feed_entry::Column::FeedId,
                                feed_entry::Column::Link,
                            ])
                            .update_columns([
                                feed_entry::Column::Title,
                                feed_entry::Column::PublishedAt,
                                feed_entry::Column::Description,
                                feed_entry::Column::Author,
                                feed_entry::Column::ThumbnailUrl,
                            ])
                            .to_owned(),
                        )
                        .exec(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let fe_models = feed_entry::Entity::find()
                        .filter(feed_entry::Column::Link.is_in(links))
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

                    find_by_id(txn, pf_id, data.profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn update_feed(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: FeedUpdateData,
    ) -> Result<Feed, Error> {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    let Some(pf_model) = profile_feed::Entity::find_by_id(id)
                        .filter(profile_feed::Column::ProfileId.eq(profile_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
                    };

                    let mut active_model = pf_model.clone().into_active_model();
                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title)
                    }
                    if let Some(folder_id) = data.folder_id {
                        active_model.folder_id.set_if_not_equals(folder_id)
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
                                profile_id: Set(profile_id),
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
                                profile_id: Set(profile_id),
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

                    find_by_id(txn, id, profile_id).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete_feed(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        let result = profile_feed::Entity::delete_by_id(id)
            .filter(profile_feed::Column::ProfileId.eq(profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(id));
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

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    filters: Option<FeedFindManyFilters>,
) -> Result<Paginated<Feed>, Error> {
    let mut query = profile_feed::Entity::find()
        .find_also_related(feed::Entity)
        .order_by_asc(SimpleExpr::FunctionCall(Func::coalesce([
            Expr::col((profile_feed::Entity, profile_feed::Column::Title)).into(),
            Expr::col((feed::Entity, feed::Column::Title)).into(),
        ])))
        .order_by_asc(profile_feed::Column::Id)
        .limit(limit.map(|e| e + 1));

    let mut conditions = Condition::all().add(profile_feed::Column::ProfileId.eq(profile_id));
    if let Some(id) = id {
        conditions = conditions.add(profile_feed::Column::Id.eq(id));
    }
    if let Some(filters) = filters {
        if let Some(tags) = filters.tags {
            query = query
                .join(
                    JoinType::InnerJoin,
                    profile_feed::Relation::ProfileFeedTag.def(),
                )
                .join(JoinType::InnerJoin, profile_feed_tag::Relation::Tag.def());

            conditions = conditions.add(tag::Column::Title.is_in(tags));
        }
    }
    if let Some(raw) = cursor_raw.as_deref() {
        let cursor = base_64::decode::<Cursor>(raw)?;

        conditions = conditions.add(
            Expr::tuple([
                Func::coalesce([
                    Expr::col((profile_feed::Entity, profile_feed::Column::Title)).into(),
                    Expr::col((feed::Entity, feed::Column::Title)).into(),
                ])
                .into(),
                Expr::col((profile_feed::Entity, profile_feed::Column::Id)).into(),
            ])
            .gt(Expr::tuple([
                Expr::value(cursor.title),
                Expr::value(cursor.id),
            ])),
        );
    }

    let models = query
        .filter(conditions)
        .all(db)
        .await
        .map(|e| {
            e.into_iter()
                .filter_map(|(pf, feed_opt)| feed_opt.map(|feed| (pf, feed)))
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))?;
    let pf_models = models.clone().into_iter().map(|e| e.0).collect::<Vec<_>>();

    let tag_models = pf_models
        .load_many_to_many(
            tag::Entity::find().order_by_asc(tag::Column::Title),
            profile_feed_tag::Entity,
            db,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let pf_ids = pf_models.iter().map(|e| e.id).collect::<Vec<_>>();

    let counts: Vec<(Uuid, i64)> = profile_feed_entry::Entity::find()
        .select_only()
        .column(profile_feed_entry::Column::ProfileFeedId)
        .column_as(profile_feed_entry::Column::Id.count(), "count")
        .filter(profile_feed_entry::Column::ProfileFeedId.is_in(pf_ids))
        .filter(profile_feed_entry::Column::HasRead.eq(false))
        .group_by(profile_feed_entry::Column::ProfileFeedId)
        .into_tuple()
        .all(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    let count_map: HashMap<Uuid, i64> = counts.into_iter().collect();

    let mut feeds = models
        .into_iter()
        .zip(tag_models.into_iter())
        .map(|((pf, feed), tags)| {
            let unread_count = count_map.get(&pf.id).cloned().unwrap_or_default();
            Feed::from(PfWithFeedAndTagsAndUnreadCount {
                pf,
                feed,
                tags,
                unread_count,
            })
        })
        .collect::<Vec<_>>();
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if feeds.len() > limit {
            feeds = feeds.into_iter().take(limit).collect();

            if let Some(last) = feeds.last() {
                let c = Cursor {
                    id: last.id,
                    title: last
                        .title
                        .to_owned()
                        .unwrap_or(last.original_title.to_owned()),
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Feed> {
        cursor,
        data: feeds,
    })
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    profile_id: Uuid,
) -> Result<Feed, Error> {
    let feeds = find(db, Some(id), profile_id, Some(1), None, None).await?;

    feeds.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
struct Cursor {
    pub id: Uuid,
    pub title: String,
}
