use async_trait::async_trait;
use colette_core::{
    common,
    feeds::{Error, FeedCreateData, FeedFindManyParams, FeedsRepository},
    Feed,
};
use colette_database::{feed_entries, profile_feed_entries, profile_feeds, FindOneParams};
use nanoid::nanoid;
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

#[async_trait]
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

        println!("{}, {}", data.profile_id, feed_id);
        let profile_feed_id = queries::profile_feeds::insert(
            &mut *tx,
            profile_feeds::InsertData {
                id: nanoid!(),
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
                profile_feed_entries::InsertData {
                    id: nanoid!(),
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
