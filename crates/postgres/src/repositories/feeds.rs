use anyhow::anyhow;
use colette_core::{
    common::{self, FindManyParams},
    feeds::{Error, FeedsCreateData, FeedsRepository, FeedsUpdateData, StreamFeed},
    Feed,
};
use colette_entities::{entries, feed_entries, feeds, profile_feed_entries, profile_feeds};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    prelude::Expr,
    sea_query::{Func, IntoCondition, OnConflict, Query},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, SelectModel, Selector, Set, TransactionError, TransactionTrait,
};
use sqlx::types::chrono::{DateTime, FixedOffset};
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
        profile_feeds::Entity::find()
            .select_only()
            .columns(PROFILE_FEED_COLUMNS)
            .columns(FEED_COLUMNS)
            .column_as(profile_feed_entries::Column::Id.count(), "unread_count")
            .join(JoinType::Join, profile_feeds::Relation::Feeds.def())
            .join(JoinType::Join, feeds::Relation::FeedEntries.def())
            .join(
                JoinType::LeftJoin,
                profile_feed_entries::Relation::FeedEntries
                    .def()
                    .rev()
                    .on_condition(|_, right| {
                        Expr::col((right, profile_feed_entries::Column::HasRead))
                            .eq(false)
                            .into_condition()
                    }),
            )
            .filter(profile_feeds::Column::ProfileId.eq(params.profile_id))
            .group_by(profile_feeds::Column::Id)
            .group_by(feeds::Column::Link)
            .group_by(feeds::Column::Title)
            .group_by(feeds::Column::Url)
            .order_by_asc(profile_feeds::Column::CustomTitle)
            .order_by_asc(feeds::Column::Title)
            .order_by_asc(profile_feeds::Column::Id)
            .into_model::<FeedSelect>()
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Feed::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Feed, Error> {
        let Some(feed) = feed_by_id(params.id, params.profile_id)
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
                    let feed_model = feeds::ActiveModel {
                        link: Set(link.clone()),
                        title: Set(data.feed.title),
                        url: Set(if data.url == link {
                            None
                        } else {
                            Some(data.url)
                        }),
                        ..Default::default()
                    };

                    feeds::Entity::insert(feed_model)
                        .on_conflict(
                            OnConflict::column(feeds::Column::Link)
                                .update_columns([feeds::Column::Title, feeds::Column::Url])
                                .to_owned(),
                        )
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(feed) = feeds::Entity::find()
                        .select_only()
                        .column(feeds::Column::Id)
                        .filter(feeds::Column::Link.eq(link))
                        .into_model::<BigIntInsert>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created feed")));
                    };

                    let profile_feed_model = profile_feeds::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        profile_id: Set(data.profile_id),
                        feed_id: Set(feed.id),
                        ..Default::default()
                    };

                    profile_feeds::Entity::insert(profile_feed_model)
                        .on_conflict(
                            OnConflict::columns([
                                profile_feeds::Column::ProfileId,
                                profile_feeds::Column::FeedId,
                            ])
                            .do_nothing_on([profile_feeds::Column::Id])
                            .to_owned(),
                        )
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(profile_feed) = profile_feeds::Entity::find()
                        .select_only()
                        .column(profile_feeds::Column::Id)
                        .filter(profile_feeds::Column::ProfileId.eq(data.profile_id))
                        .filter(profile_feeds::Column::FeedId.eq(feed.id))
                        .into_model::<UuidInsert>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!(
                            "Failed to fetch created profile feed"
                        )));
                    };

                    for e in data.feed.entries {
                        let link = e.link.to_string();
                        let entry_model = entries::ActiveModel {
                            link: Set(link.clone()),
                            title: Set(e.title),
                            published_at: Set(e.published.map(|e| e.into())),
                            description: Set(e.description),
                            author: Set(e.author),
                            thumbnail_url: Set(e.thumbnail.map(String::from)),
                            ..Default::default()
                        };

                        entries::Entity::insert(entry_model)
                            .on_conflict(
                                OnConflict::column(entries::Column::Link)
                                    .update_columns([
                                        entries::Column::Title,
                                        entries::Column::PublishedAt,
                                        entries::Column::Description,
                                        entries::Column::Author,
                                        entries::Column::ThumbnailUrl,
                                    ])
                                    .to_owned(),
                            )
                            .exec_without_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let Some(entry) = entries::Entity::find()
                            .select_only()
                            .column(entries::Column::Id)
                            .filter(entries::Column::Link.eq(link))
                            .into_model::<BigIntInsert>()
                            .one(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                        else {
                            return Err(Error::Unknown(anyhow!("Failed to fetch created entry")));
                        };

                        let feed_entry_model = feed_entries::ActiveModel {
                            feed_id: Set(feed.id),
                            entry_id: Set(entry.id),
                            ..Default::default()
                        };

                        feed_entries::Entity::insert(feed_entry_model)
                            .on_conflict(
                                OnConflict::columns([
                                    feed_entries::Column::FeedId,
                                    feed_entries::Column::EntryId,
                                ])
                                .do_nothing_on([feed_entries::Column::Id])
                                .to_owned(),
                            )
                            .exec_without_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let Some(feed_entry) = feed_entries::Entity::find()
                            .select_only()
                            .column(feed_entries::Column::Id)
                            .filter(feed_entries::Column::FeedId.eq(feed.id))
                            .filter(feed_entries::Column::EntryId.eq(entry.id))
                            .into_model::<BigIntInsert>()
                            .one(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                        else {
                            return Err(Error::Unknown(anyhow!(
                                "Failed to fetch created feed entry"
                            )));
                        };

                        let profile_feed_entry = profile_feed_entries::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            profile_feed_id: Set(profile_feed.id),
                            feed_entry_id: Set(feed_entry.id),
                            ..Default::default()
                        };

                        profile_feed_entries::Entity::insert(profile_feed_entry)
                            .on_conflict(
                                OnConflict::columns([
                                    profile_feed_entries::Column::ProfileFeedId,
                                    profile_feed_entries::Column::FeedEntryId,
                                ])
                                .do_nothing_on([profile_feed_entries::Column::Id])
                                .to_owned(),
                            )
                            .exec_without_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    let Some(feed) = feed_by_id(profile_feed.id, data.profile_id)
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
                    let mut model = profile_feeds::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if data.custom_title.is_some() {
                        model.custom_title = Set(data.custom_title)
                    }

                    profile_feeds::Entity::update(model)
                        .filter(profile_feeds::Column::ProfileId.eq(params.profile_id))
                        .exec(txn)
                        .await
                        .map_err(|e| match e {
                            DbErr::RecordNotFound(_) | DbErr::RecordNotUpdated => {
                                Error::NotFound(params.id)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    let Some(feed) = feed_by_id(params.id, params.profile_id)
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
        let result = profile_feeds::Entity::delete_by_id(params.id)
            .filter(profile_feeds::Column::ProfileId.eq(params.profile_id))
            .exec(&self.db)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<Result<StreamFeed, Error>>, Error> {
        feeds::Entity::find()
            .select_only()
            .column(feeds::Column::Id)
            .expr_as(
                Func::coalesce([
                    Expr::col(feeds::Column::Url).into(),
                    Expr::col(feeds::Column::Link).into(),
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
                        .from(profile_feed_entries::Entity)
                        .and_where(
                            Expr::col((
                                profile_feed_entries::Entity,
                                profile_feed_entries::Column::FeedEntryId,
                            ))
                            .equals((feed_entries::Entity, feed_entries::Column::Id)),
                        )
                        .to_owned();

                    let result = feed_entries::Entity::delete_many()
                        .filter(Expr::exists(subquery).not())
                        .exec(txn)
                        .await?;

                    println!("Deleted {} orphaned feed entries", result.rows_affected);

                    let subquery = Query::select()
                        .from(feed_entries::Entity)
                        .and_where(
                            Expr::col((feed_entries::Entity, feed_entries::Column::EntryId))
                                .equals((entries::Entity, entries::Column::Id)),
                        )
                        .to_owned();

                    let result = entries::Entity::delete_many()
                        .filter(Expr::exists(subquery).not())
                        .exec(txn)
                        .await?;

                    println!("Deleted {} orphaned entries", result.rows_affected);

                    let subquery = Query::select()
                        .from(profile_feeds::Entity)
                        .and_where(
                            Expr::col((profile_feeds::Entity, profile_feeds::Column::FeedId))
                                .equals((feeds::Entity, feeds::Column::Id)),
                        )
                        .to_owned();

                    let result = feeds::Entity::delete_many()
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

#[derive(sea_orm::FromQueryResult)]
struct FeedSelect {
    id: Uuid,
    link: String,
    title: String,
    url: Option<String>,
    custom_title: Option<String>,
    profile_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    unread_count: Option<i64>,
}

impl From<FeedSelect> for Feed {
    fn from(value: FeedSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            custom_title: value.custom_title,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
            unread_count: value.unread_count,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct BigIntInsert {
    id: i64,
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct UuidInsert {
    id: Uuid,
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct StreamSelect {
    pub id: i64,
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

const PROFILE_FEED_COLUMNS: [profile_feeds::Column; 5] = [
    profile_feeds::Column::Id,
    profile_feeds::Column::CustomTitle,
    profile_feeds::Column::ProfileId,
    profile_feeds::Column::CreatedAt,
    profile_feeds::Column::UpdatedAt,
];

const FEED_COLUMNS: [feeds::Column; 3] = [
    feeds::Column::Link,
    feeds::Column::Title,
    feeds::Column::Url,
];

fn feed_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<FeedSelect>> {
    profile_feeds::Entity::find_by_id(id)
        .select_only()
        .columns(PROFILE_FEED_COLUMNS)
        .columns(FEED_COLUMNS)
        .column_as(profile_feed_entries::Column::Id.count(), "unread_count")
        .join(JoinType::Join, profile_feeds::Relation::Feeds.def())
        .join(JoinType::Join, feeds::Relation::FeedEntries.def())
        .join(
            JoinType::LeftJoin,
            profile_feed_entries::Relation::FeedEntries
                .def()
                .rev()
                .on_condition(|_, right| {
                    Expr::col((right, profile_feed_entries::Column::HasRead))
                        .eq(false)
                        .into_condition()
                }),
        )
        .filter(profile_feeds::Column::ProfileId.eq(profile_id))
        .group_by(profile_feeds::Column::Id)
        .group_by(feeds::Column::Link)
        .group_by(feeds::Column::Title)
        .group_by(feeds::Column::Url)
        .into_model::<FeedSelect>()
}
