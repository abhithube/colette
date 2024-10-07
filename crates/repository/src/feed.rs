use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    feed::{
        Cursor, Error, FeedCacheData, FeedCreateData, FeedFindManyFilters, FeedRepository,
        FeedUpdateData, ProcessedFeed,
    },
    Feed,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    prelude::Uuid,
    sqlx::{self, PgExecutor},
    ConnectionTrait, DatabaseConnection, DbErr, TransactionError, TransactionTrait,
};

use crate::query;

pub struct FeedSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl FeedSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for FeedSqlRepository {
    type Params = IdParams;
    type Output = Result<Feed, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(self.db.get_postgres_connection_pool(), params).await
    }
}

#[async_trait::async_trait]
impl Creatable for FeedSqlRepository {
    type Data = FeedCreateData;
    type Output = Result<Feed, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = self
            .db
            .transaction::<_, Uuid, Error>(|txn| {
                Box::pin(async move {
                    let feed_id = if let Some(feed) = data.feed {
                        create_feed_with_entries(txn, data.url, feed).await?
                    } else {
                        let Some(feed_model) = query::feed::select_by_url(txn, data.url.clone())
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                        else {
                            return Err(Error::Conflict(data.url));
                        };

                        feed_model.id
                    };

                    let pf_id = match query::profile_feed::insert(
                        txn,
                        Uuid::new_v4(),
                        Some(data.pinned),
                        data.profile_id,
                        feed_id,
                    )
                    .await
                    {
                        Ok(result) => Ok(result.last_insert_id),
                        Err(DbErr::RecordNotInserted) => {
                            let Some(model) = query::profile_feed::select_by_unique_index(
                                txn,
                                data.profile_id,
                                feed_id,
                            )
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

                    let fe_models = query::feed_entry::select_many_by_feed_id(txn, feed_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    let insert_many = fe_models
                        .into_iter()
                        .map(|e| query::profile_feed_entry::InsertMany {
                            id: Uuid::new_v4(),
                            feed_entry_id: e.id,
                        })
                        .collect::<Vec<_>>();

                    query::profile_feed_entry::insert_many(
                        txn,
                        insert_many,
                        pf_id,
                        data.profile_id,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(pf_id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        if let Some(tags) = data.tags {
            link_tags(&self.db, id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        find_by_id(
            self.db.get_postgres_connection_pool(),
            IdParams::new(id, data.profile_id),
        )
        .await
    }
}

#[async_trait::async_trait]
impl Updatable for FeedSqlRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<Feed, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        colette_postgres::profile_feed::update(
            &mut *tx,
            params.id,
            params.profile_id,
            data.title,
            data.pinned,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        if let Some(tags) = data.tags {
            link_tags(&self.db, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = find_by_id(&mut *tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }
}

#[async_trait::async_trait]
impl Deletable for FeedSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        colette_postgres::profile_feed::delete(
            self.db.get_postgres_connection_pool(),
            params.id,
            params.profile_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl FeedRepository for FeedSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Vec<Feed>, Error> {
        find(
            self.db.get_postgres_connection_pool(),
            None,
            profile_id,
            limit,
            cursor,
            filters,
        )
        .await
    }

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    create_feed_with_entries(txn, data.url, data.feed).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    async fn stream(&self) -> Result<BoxStream<Result<(i32, String), Error>>, Error> {
        query::feed::stream(&self.db)
            .await
            .map(|e| {
                e.map(|e| e.map_err(|e| Error::Unknown(e.into())))
                    .map_err(|e| Error::Unknown(e.into()))
                    .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

pub(crate) async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedFindManyFilters>,
) -> Result<Vec<Feed>, Error> {
    let mut pinned: Option<bool> = None;
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        pinned = filters.pinned;
        tags = filters.tags;
    }

    colette_postgres::profile_feed::find(executor, id, profile_id, pinned, tags, cursor, limit)
        .await
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(executor: impl PgExecutor<'_>, params: IdParams) -> Result<Feed, Error> {
    let mut feeds = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if feeds.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feeds.swap_remove(0))
}

async fn create_feed_with_entries<Db: ConnectionTrait>(
    db: &Db,
    url: String,
    feed: ProcessedFeed,
) -> Result<i32, Error> {
    let link = feed.link.to_string();
    let url = if url == link { None } else { Some(url) };

    let result = query::feed::insert(db, link, feed.title, url)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;
    let feed_id = result.last_insert_id;

    let insert_many = feed
        .entries
        .into_iter()
        .map(|e| query::feed_entry::InsertMany {
            link: e.link.to_string(),
            title: e.title,
            published_at: e.published.into(),
            description: e.description,
            author: e.author,
            thumbnail_url: e.thumbnail.map(String::from),
        })
        .collect::<Vec<_>>();

    query::feed_entry::insert_many(db, insert_many, feed_id)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(feed_id)
}

pub(crate) async fn link_tags(
    db: &DatabaseConnection,
    profile_feed_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        return colette_postgres::profile_feed_tag::delete_many_in_titles(
            db.get_postgres_connection_pool(),
            &tags.data,
            profile_id,
        )
        .await;
    }

    let mut tx = db.get_postgres_connection_pool().begin().await?;

    if let TagsLinkAction::Set = tags.action {
        colette_postgres::profile_feed_tag::delete_many_not_in_titles(
            &mut *tx, &tags.data, profile_id,
        )
        .await?;
    }

    colette_postgres::tag::insert_many(
        &mut *tx,
        tags.data
            .iter()
            .map(|e| colette_postgres::tag::InsertMany {
                id: Uuid::new_v4(),
                title: e.to_owned(),
            })
            .collect(),
        profile_id,
    )
    .await?;

    let tag_ids =
        colette_postgres::tag::select_ids_by_titles(&mut *tx, &tags.data, profile_id).await?;

    let insert_many = tag_ids
        .into_iter()
        .map(|e| colette_postgres::profile_feed_tag::InsertMany {
            profile_feed_id,
            tag_id: e,
        })
        .collect::<Vec<_>>();

    colette_postgres::profile_feed_tag::insert_many(&mut *tx, insert_many, profile_id).await?;

    tx.commit().await
}
