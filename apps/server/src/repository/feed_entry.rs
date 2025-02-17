use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry,
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::common::DbUrl;

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
        let feed_entries = sqlx::query_file_as!(
            FeedEntryRow,
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
        .map(|e| e.into_iter().map(FeedEntry::from).collect())?;

        Ok(feed_entries)
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
                _ => Error::Database(e),
            })?;
        }

        Ok(())
    }
}

impl FeedEntryRepository for PostgresFeedEntryRepository {}

struct FeedEntryRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<DbUrl>,
    has_read: bool,
    feed_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<FeedEntryRow> for FeedEntry {
    fn from(value: FeedEntryRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            has_read: value.has_read,
            feed_id: value.feed_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
