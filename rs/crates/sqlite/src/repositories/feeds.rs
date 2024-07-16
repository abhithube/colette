use async_trait::async_trait;
use colette_core::{
    common::{self, SendableStream},
    feeds::{Error, FeedCreateData, FeedFindManyParams, FeedUpdateData, FeedsRepository},
    Feed,
};
use colette_database::{feed_entries, profile_feeds::UpdateParams, FindOneParams};
use futures::TryStreamExt;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::queries;

pub struct FeedsSqliteRepository {
    pool: SqlitePool,
}

impl FeedsSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FeedsRepository for FeedsSqliteRepository {
    async fn find_many(&self, params: FeedFindManyParams) -> Result<Vec<Feed>, Error> {
        let feeds = queries::profile_feeds::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(feeds)
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Feed, Error> {
        let feed = queries::profile_feeds::select_by_id(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(feed)
    }

    async fn create(&self, data: FeedCreateData) -> Result<Feed, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = queries::feeds::insert(&mut *tx, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let result = queries::profile_feeds::insert(
            &mut *tx,
            queries::profile_feeds::InsertParams {
                id: Uuid::new_v4(),
                profile_id: &data.profile_id,
                feed_id,
            },
        )
        .await;

        let profile_feed_id = match result {
            Ok(id) => id,
            Err(_) => queries::profile_feeds::select(
                &mut *tx,
                queries::profile_feeds::SelectParams {
                    profile_id: &data.profile_id,
                    feed_id,
                },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?,
        };

        for e in data.feed.entries {
            let entry_id = queries::entries::insert(&mut *tx, (&e).into())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let result = queries::feed_entries::insert(
                &mut *tx,
                feed_entries::InsertParams { feed_id, entry_id },
            )
            .await;

            let feed_entry_id = match result {
                Ok(id) => id,
                Err(_) => queries::feed_entries::select(
                    &mut *tx,
                    queries::feed_entries::SelectParams { feed_id, entry_id },
                )
                .await
                .map_err(|e| Error::Unknown(e.into()))?,
            };

            queries::profile_feed_entries::insert(
                &mut *tx,
                queries::profile_feed_entries::InsertParams {
                    id: Uuid::new_v4(),
                    profile_feed_id: &profile_feed_id,
                    feed_entry_id,
                },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        }

        let feed = queries::profile_feeds::select_by_id(
            &mut *tx,
            FindOneParams {
                id: &profile_feed_id,
                profile_id: &data.profile_id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }

    async fn update(
        &self,
        params: common::FindOneParams,
        data: FeedUpdateData,
    ) -> Result<Feed, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        queries::profile_feeds::update(
            &mut *tx,
            UpdateParams {
                id: &params.id,
                profile_id: &params.profile_id,
                custom_title: data.custom_title.as_deref(),
            },
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        let feed = queries::profile_feeds::select_by_id(&mut *tx, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(feed)
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        queries::profile_feeds::delete(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }

    fn iterate(&self) -> SendableStream<Result<(i64, String), Error>> {
        Box::pin(queries::feeds::iterate(&self.pool).map_err(|e| Error::Unknown(e.into())))
    }
}
