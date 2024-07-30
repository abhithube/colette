use anyhow::anyhow;
use colette_core::{
    common::{self, FindManyParams, UpdateTagList},
    feeds::{Error, FeedsCreateData, FeedsRepository, FeedsUpdateData, StreamFeed},
    Feed,
};
use colette_entities::{
    entry, feed, feed_entry, profile_feed, profile_feed_entry, profile_feed_tag, PartialFeed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    prelude::Expr,
    sea_query::{Func, IntoCondition, OnConflict, Query},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Select, Set, TransactionError, TransactionTrait,
};
use uuid::Uuid;

pub struct FeedsSqlRepository {
    db: DatabaseConnection,
}

impl FeedsSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedsRepository for FeedsSqlRepository {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Feed>, Error> {
        select(None, params.profile_id)
            .order_by_asc(feed::Column::Title)
            .order_by_asc(profile_feed::Column::Id)
            .into_partial_model::<PartialFeed>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Feed::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Feed, Error> {
        let Some(feed) = select(Some(params.id), params.profile_id)
            .into_partial_model::<PartialFeed>()
            .one(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(feed.into())
    }

    async fn create(&self, data: FeedsCreateData) -> Result<Feed, Error> {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    let link = data.feed.link.to_string();
                    let feed_model = feed::ActiveModel {
                        link: Set(link.clone()),
                        title: Set(data.feed.title),
                        url: Set(if data.url == link {
                            None
                        } else {
                            Some(data.url)
                        }),
                        ..Default::default()
                    };

                    let feed_model = feed::Entity::insert(feed_model)
                        .on_conflict(
                            OnConflict::column(feed::Column::Link)
                                .update_columns([feed::Column::Title, feed::Column::Url])
                                .to_owned(),
                        )
                        .exec_with_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let pf_model = profile_feed::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        profile_id: Set(data.profile_id),
                        feed_id: Set(feed_model.id),
                        ..Default::default()
                    };

                    let pf_model = match profile_feed::Entity::insert(pf_model)
                        .on_conflict(
                            OnConflict::columns([
                                profile_feed::Column::ProfileId,
                                profile_feed::Column::FeedId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec_with_returning(txn)
                        .await
                    {
                        Ok(model) => Ok(model),
                        Err(DbErr::RecordNotFound(_)) => {
                            let Some(model) = profile_feed::Entity::find()
                                .filter(profile_feed::Column::ProfileId.eq(data.profile_id))
                                .filter(profile_feed::Column::FeedId.eq(feed_model.id))
                                .one(txn)
                                .await
                                .map_err(|e| Error::Unknown(e.into()))?
                            else {
                                return Err(Error::Unknown(anyhow!(
                                    "Failed to fetch created profile feed"
                                )));
                            };

                            Ok(model)
                        }
                        _ => Err(Error::Unknown(anyhow!("Failed to create profile feed"))),
                    }?;

                    for e in data.feed.entries {
                        let link = e.link.to_string();
                        let entry_model = entry::ActiveModel {
                            link: Set(link.clone()),
                            title: Set(e.title),
                            published_at: Set(e.published.map(|e| e.into())),
                            description: Set(e.description),
                            author: Set(e.author),
                            thumbnail_url: Set(e.thumbnail.map(String::from)),
                            ..Default::default()
                        };

                        let entry_model = entry::Entity::insert(entry_model)
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
                            .exec_with_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let fe_model = feed_entry::ActiveModel {
                            feed_id: Set(feed_model.id),
                            entry_id: Set(entry_model.id),
                            ..Default::default()
                        };

                        let fe_model = match feed_entry::Entity::insert(fe_model)
                            .on_conflict(
                                OnConflict::columns([
                                    feed_entry::Column::FeedId,
                                    feed_entry::Column::EntryId,
                                ])
                                .do_nothing()
                                .to_owned(),
                            )
                            .exec_with_returning(txn)
                            .await
                        {
                            Ok(model) => Ok(model),
                            Err(DbErr::RecordNotFound(_)) => {
                                let Some(model) = feed_entry::Entity::find()
                                    .filter(feed_entry::Column::FeedId.eq(feed_model.id))
                                    .filter(feed_entry::Column::EntryId.eq(entry_model.id))
                                    .one(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?
                                else {
                                    return Err(Error::Unknown(anyhow!(
                                        "Failed to fetch created feed entry"
                                    )));
                                };

                                Ok(model)
                            }
                            _ => Err(Error::Unknown(anyhow!("Failed to create feed entry"))),
                        }?;

                        let pfe_model = profile_feed_entry::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            profile_feed_id: Set(pf_model.id),
                            feed_entry_id: Set(fe_model.id),
                            ..Default::default()
                        };

                        profile_feed_entry::Entity::insert(pfe_model)
                            .on_conflict(
                                OnConflict::columns([
                                    profile_feed_entry::Column::ProfileFeedId,
                                    profile_feed_entry::Column::FeedEntryId,
                                ])
                                .do_nothing()
                                .to_owned(),
                            )
                            .exec_without_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    let Some(feed) = select(Some(pf_model.id), data.profile_id)
                        .into_partial_model::<PartialFeed>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created feed")));
                    };

                    Ok(feed.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn update(
        &self,
        params: common::FindOneParams,
        data: FeedsUpdateData,
    ) -> Result<Feed, Error> {
        self.db
            .transaction::<_, Feed, Error>(|txn| {
                Box::pin(async move {
                    if let Some(tags) = data.tags {
                        match tags {
                            UpdateTagList::Add(tag_ids) => {
                                let models = tag_ids
                                    .into_iter()
                                    .map(|id| profile_feed_tag::ActiveModel {
                                        tag_id: Set(id),
                                        profile_feed_id: Set(params.id),
                                    })
                                    .collect::<Vec<_>>();

                                profile_feed_tag::Entity::insert_many(models)
                                    .on_conflict(
                                        OnConflict::columns([
                                            profile_feed_tag::Column::ProfileFeedId,
                                            profile_feed_tag::Column::TagId,
                                        ])
                                        .do_nothing()
                                        .to_owned(),
                                    )
                                    .exec_without_returning(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;
                            }
                            UpdateTagList::Remove(tag_ids) => {
                                profile_feed_tag::Entity::delete_many()
                                    .filter(profile_feed_tag::Column::ProfileFeedId.eq(params.id))
                                    .filter(profile_feed_tag::Column::TagId.is_in(tag_ids))
                                    .exec(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;
                            }
                            UpdateTagList::Set(tag_ids) => {
                                profile_feed_tag::Entity::delete_many()
                                    .filter(profile_feed_tag::Column::ProfileFeedId.eq(params.id))
                                    .exec(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;

                                let models = tag_ids
                                    .into_iter()
                                    .map(|id| profile_feed_tag::ActiveModel {
                                        tag_id: Set(id),
                                        profile_feed_id: Set(params.id),
                                    })
                                    .collect::<Vec<_>>();

                                profile_feed_tag::Entity::insert_many(models)
                                    .exec_without_returning(txn)
                                    .await
                                    .map_err(|e| Error::Unknown(e.into()))?;
                            }
                        }
                    }

                    let Some(feed) = select(Some(params.id), params.profile_id)
                        .into_partial_model::<PartialFeed>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch updated feed")));
                    };

                    Ok(feed.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
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

    async fn stream(&self) -> Result<BoxStream<Result<StreamFeed, Error>>, Error> {
        feed::Entity::find()
            .select_only()
            .column(feed::Column::Id)
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

    async fn cleanup(&self) -> Result<(), Error> {
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

                    println!("Deleted {} orphaned feed entries", result.rows_affected);

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

                    println!("Deleted {} orphaned entries", result.rows_affected);

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

                    println!("Deleted {} orphaned feeds", result.rows_affected);

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
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

fn select(id: Option<Uuid>, profile_id: Uuid) -> Select<profile_feed::Entity> {
    let query = match id {
        Some(id) => profile_feed::Entity::find_by_id(id),
        None => profile_feed::Entity::find(),
    };

    query
        .column_as(profile_feed_entry::Column::Id.count(), "unread_count")
        .join(JoinType::Join, profile_feed::Relation::Feed.def())
        .join(
            JoinType::LeftJoin,
            profile_feed_entry::Relation::ProfileFeed
                .def()
                .rev()
                .on_condition(|_, right| {
                    Expr::col((right, profile_feed_entry::Column::HasRead))
                        .eq(false)
                        .into_condition()
                }),
        )
        .filter(profile_feed::Column::ProfileId.eq(profile_id))
        .group_by(profile_feed::Column::Id)
        .group_by(feed::Column::Link)
        .group_by(feed::Column::Title)
        .group_by(feed::Column::Url)
}
