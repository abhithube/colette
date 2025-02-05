use colette_core::{
    Feed,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed::{
        ConflictError, Error, FeedCacheData, FeedCreateData, FeedFindParams, FeedRepository,
        FeedUpdateData,
    },
};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

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
        let feeds = crate::common::select_feeds(
            &self.pool,
            params.id,
            params.folder_id,
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

        let feed_id = sqlx::query_file_scalar!("queries/feeds/select_by_url.sql", data.url)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    Error::Conflict(ConflictError::NotCached(data.url.clone()))
                }
                _ => Error::Database(e),
            })?;

        let uf_id = sqlx::query_file_scalar!(
            "queries/user_feeds/insert.sql",
            data.title,
            data.folder_id,
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
                    uf_id,
                    data.user_id,
                    &tags.into_iter().map(String::from).collect::<Vec<_>>(),
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

        if data.title.is_some() || data.folder_id.is_some() {
            let (has_folder, folder_id) = match data.folder_id {
                Some(folder_id) => (true, folder_id),
                None => (false, None),
            };

            sqlx::query_file!(
                "queries/user_feeds/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title,
                has_folder,
                folder_id
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
                    params.id,
                    params.user_id,
                    &tags.into_iter().map(String::from).collect::<Vec<_>>(),
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
        sqlx::query_file!("queries/user_feeds/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn cache(&self, data: FeedCacheData) -> Result<(), Error> {
        crate::common::insert_feed_with_entries(&self.pool, data.url, data.feed).await?;

        Ok(())
    }

    fn stream(&self) -> BoxStream<Result<String, Error>> {
        sqlx::query_file_scalar!("queries/feeds/stream.sql")
            .fetch(&self.pool)
            .map_err(Error::Database)
            .boxed()
    }
}
