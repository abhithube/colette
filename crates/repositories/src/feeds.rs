use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use colette_core::{
    common::{self, FindManyParams, UpdateTagList},
    feeds::{Error, FeedsCreateData, FeedsRepository, FeedsUpdateData, StreamFeed},
    Feed,
};
use colette_entities::{
    entry, feed, feed_entry, profile_feed, profile_feed_entry, profile_feed_tag,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    prelude::Expr,
    sea_query::{Func, IntoCondition, OnConflict, Query},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, SelectModel, Selector, Set, TransactionError, TransactionTrait,
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
        profile_feed::Entity::find()
            .select_only()
            .columns(PROFILE_FEED_COLUMNS)
            .columns(FEED_COLUMNS)
            .column_as(profile_feed_entry::Column::Id.count(), "unread_count")
            .join(JoinType::Join, profile_feed::Relation::Feed.def())
            .join(JoinType::Join, feed::Relation::FeedEntry.def())
            .join(
                JoinType::LeftJoin,
                profile_feed_entry::Relation::FeedEntry
                    .def()
                    .rev()
                    .on_condition(|_, right| {
                        Expr::col((right, profile_feed_entry::Column::HasRead))
                            .eq(false)
                            .into_condition()
                    }),
            )
            .filter(profile_feed::Column::ProfileId.eq(params.profile_id))
            .group_by(profile_feed::Column::Id)
            .group_by(feed::Column::Link)
            .group_by(feed::Column::Title)
            .group_by(feed::Column::Url)
            .order_by_asc(profile_feed::Column::CustomTitle)
            .order_by_asc(feed::Column::Title)
            .order_by_asc(profile_feed::Column::Id)
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

                    feed::Entity::insert(feed_model)
                        .on_conflict(
                            OnConflict::column(feed::Column::Link)
                                .update_columns([feed::Column::Title, feed::Column::Url])
                                .to_owned(),
                        )
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(feed) = feed::Entity::find()
                        .select_only()
                        .column(feed::Column::Id)
                        .filter(feed::Column::Link.eq(link))
                        .into_model::<IntInsert>()
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::Unknown(anyhow!("Failed to fetch created feed")));
                    };

                    let profile_feed_model = profile_feed::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        profile_id: Set(data.profile_id),
                        feed_id: Set(feed.id),
                        ..Default::default()
                    };

                    profile_feed::Entity::insert(profile_feed_model)
                        .on_conflict(
                            OnConflict::columns([
                                profile_feed::Column::ProfileId,
                                profile_feed::Column::FeedId,
                            ])
                            .do_nothing_on([profile_feed::Column::Id])
                            .to_owned(),
                        )
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let Some(profile_feed) = profile_feed::Entity::find()
                        .select_only()
                        .column(profile_feed::Column::Id)
                        .filter(profile_feed::Column::ProfileId.eq(data.profile_id))
                        .filter(profile_feed::Column::FeedId.eq(feed.id))
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
                        let entry_model = entry::ActiveModel {
                            link: Set(link.clone()),
                            title: Set(e.title),
                            published_at: Set(e.published.map(|e| e.into())),
                            description: Set(e.description),
                            author: Set(e.author),
                            thumbnail_url: Set(e.thumbnail.map(String::from)),
                            ..Default::default()
                        };

                        entry::Entity::insert(entry_model)
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
                            .exec_without_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let Some(entry) = entry::Entity::find()
                            .select_only()
                            .column(entry::Column::Id)
                            .filter(entry::Column::Link.eq(link))
                            .into_model::<IntInsert>()
                            .one(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                        else {
                            return Err(Error::Unknown(anyhow!("Failed to fetch created entry")));
                        };

                        let feed_entry_model = feed_entry::ActiveModel {
                            feed_id: Set(feed.id),
                            entry_id: Set(entry.id),
                            ..Default::default()
                        };

                        feed_entry::Entity::insert(feed_entry_model)
                            .on_conflict(
                                OnConflict::columns([
                                    feed_entry::Column::FeedId,
                                    feed_entry::Column::EntryId,
                                ])
                                .do_nothing_on([feed_entry::Column::Id])
                                .to_owned(),
                            )
                            .exec_without_returning(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;

                        let Some(feed_entry) = feed_entry::Entity::find()
                            .select_only()
                            .column(feed_entry::Column::Id)
                            .filter(feed_entry::Column::FeedId.eq(feed.id))
                            .filter(feed_entry::Column::EntryId.eq(entry.id))
                            .into_model::<IntInsert>()
                            .one(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                        else {
                            return Err(Error::Unknown(anyhow!(
                                "Failed to fetch created feed entry"
                            )));
                        };

                        let profile_feed_entry = profile_feed_entry::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            profile_feed_id: Set(profile_feed.id),
                            feed_entry_id: Set(feed_entry.id),
                            ..Default::default()
                        };

                        profile_feed_entry::Entity::insert(profile_feed_entry)
                            .on_conflict(
                                OnConflict::columns([
                                    profile_feed_entry::Column::ProfileFeedId,
                                    profile_feed_entry::Column::FeedEntryId,
                                ])
                                .do_nothing_on([profile_feed_entry::Column::Id])
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
                    let mut model = profile_feed::ActiveModel {
                        id: Set(params.id),
                        ..Default::default()
                    };
                    if data.custom_title.is_some() {
                        model.custom_title = Set(data.custom_title)
                    }

                    profile_feed::Entity::update(model)
                        .filter(profile_feed::Column::ProfileId.eq(params.profile_id))
                        .exec(txn)
                        .await
                        .map_err(|e| match e {
                            DbErr::RecordNotFound(_) | DbErr::RecordNotUpdated => {
                                Error::NotFound(params.id)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

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
struct IntInsert {
    id: i32,
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
struct UuidInsert {
    id: Uuid,
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

const PROFILE_FEED_COLUMNS: [profile_feed::Column; 5] = [
    profile_feed::Column::Id,
    profile_feed::Column::CustomTitle,
    profile_feed::Column::ProfileId,
    profile_feed::Column::CreatedAt,
    profile_feed::Column::UpdatedAt,
];

const FEED_COLUMNS: [feed::Column; 3] =
    [feed::Column::Link, feed::Column::Title, feed::Column::Url];

fn feed_by_id(id: Uuid, profile_id: Uuid) -> Selector<SelectModel<FeedSelect>> {
    profile_feed::Entity::find_by_id(id)
        .select_only()
        .columns(PROFILE_FEED_COLUMNS)
        .columns(FEED_COLUMNS)
        .column_as(profile_feed_entry::Column::Id.count(), "unread_count")
        .join(JoinType::Join, profile_feed::Relation::Feed.def())
        .join(JoinType::Join, feed::Relation::FeedEntry.def())
        .join(
            JoinType::LeftJoin,
            profile_feed_entry::Relation::FeedEntry
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
        .into_model::<FeedSelect>()
}
