use colette_core::{
    Feed,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        ConflictError, Error, FeedCreateData, FeedFindParams, FeedRepository, FeedScrapedData,
        FeedUpdateData,
    },
};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::common;
use crate::repository::common::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: Pool<Postgres>,
}

impl PostgresFeedRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedRepository {
    type Params = FeedFindParams;
    type Output = Result<Vec<Feed>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let feeds = common::select_feeds(
            &self.pool,
            params.id,
            params.user_id,
            params.cursor,
            params.limit,
            params.tags,
        )
        .await?;

        Ok(feeds)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresFeedRepository {
    type Data = FeedCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self.pool.begin().await?;

        let feed_id = sqlx::query_file_scalar!(
            "queries/feeds/select_by_url.sql",
            DbUrl(data.url.clone()) as DbUrl
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(ConflictError::NotCached(data.url.clone())),
            _ => Error::Database(e),
        })?;

        let uf_id = sqlx::query_file_scalar!(
            "queries/user_feeds/insert.sql",
            data.title,
            feed_id,
            data.user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => {
                Error::Conflict(ConflictError::AlreadyExists(data.url))
            }
            _ => Error::Database(e),
        })?;

        sqlx::query_file!("queries/user_feed_entries/insert_many.sql", feed_id)
            .execute(&mut *tx)
            .await?;

        if let Some(tags) = data.tags {
            if !tags.is_empty() {
                sqlx::query_file_scalar!(
                    "queries/user_feed_tags/link.sql",
                    &tags,
                    data.user_id,
                    uf_id,
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(uf_id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFeedRepository {
    type Params = IdParams;
    type Data = FeedUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self.pool.begin().await?;

        if data.title.is_some() {
            sqlx::query_file!(
                "queries/user_feeds/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;
        }

        if let Some(tags) = data.tags {
            if !tags.is_empty() {
                sqlx::query_file_scalar!(
                    "queries/user_feed_tags/link.sql",
                    &tags,
                    params.user_id,
                    params.id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresFeedRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = sqlx::query_file!("queries/user_feeds/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn save_scraped(&self, data: FeedScrapedData) -> Result<(), Error> {
        if data.link_to_users {
            let mut tx = self.pool.begin().await?;

            let feed_id = common::insert_feed_with_entries(&mut *tx, data.url, data.feed).await?;

            sqlx::query_file!("queries/user_feed_entries/insert_many.sql", feed_id)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
        } else {
            common::insert_feed_with_entries(&self.pool, data.url, data.feed).await?;
        }

        Ok(())
    }

    fn stream_urls(&self) -> BoxStream<Result<String, Error>> {
        sqlx::query_file_scalar!("queries/feeds/stream.sql")
            .fetch(&self.pool)
            .map_err(Error::Database)
            .boxed()
    }
}
