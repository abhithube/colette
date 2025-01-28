use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: Pool<Postgres>,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedEntryRepository {
    type Params = FeedEntryFindParams;
    type Output = Result<Vec<FeedEntry>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        sqlx::query_file_as!(
            FeedEntry,
            "queries/user_feed_entries/select.sql",
            params.tags.is_some(),
            &params.tags.unwrap_or_default(),
            params.user_id,
            params.id.is_none(),
            params.id,
            params.feed_id.is_none(),
            params.feed_id,
            params.has_read.is_none(),
            params.has_read,
            params.cursor.is_none(),
            params.cursor.as_ref().map(|e| e.published_at),
            params.cursor.map(|e| e.id),
            params.limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.has_read.is_some() {
            sqlx::query_file!(
                "queries/user_feed_entries/update.sql",
                params.id,
                params.user_id,
                data.has_read.is_some(),
                data.has_read
            )
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;
        }

        Ok(())
    }
}

impl FeedEntryRepository for PostgresFeedEntryRepository {}
