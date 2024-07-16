use colette_core::{
    common,
    feeds::{Error, FeedCreateData, FeedFindManyParams, FeedUpdateData, FeedsRepository},
    Feed,
};
use colette_database::{feed_entries, FindOneParams};
use sqlx::PgPool;

use crate::queries;

#[derive(Clone)]
pub struct FeedsPostgresRepository {
    pool: PgPool,
}

impl FeedsPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedsRepository for FeedsPostgresRepository {
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

        let profile_feed_id = queries::profile_feeds::insert(
            &mut *tx,
            queries::profile_feeds::InsertData {
                profile_id: &data.profile_id,
                feed_id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        for e in data.feed.entries {
            let entry_id = queries::entries::insert(&mut *tx, (&e).into())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let feed_entry_id = queries::feed_entries::insert(
                &mut *tx,
                feed_entries::InsertData { feed_id, entry_id },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

            queries::profile_feed_entries::insert(
                &mut *tx,
                queries::profile_feed_entries::InsertData {
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
        let feed = queries::profile_feeds::update(&self.pool, (&params).into(), (&data).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

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
}