use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_handler::{FeedEntryDto, FeedEntryQueryParams, FeedEntryQueryRepository};
use sqlx::PgPool;
use uuid::Uuid;

use crate::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: PgPool,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FeedEntryQueryRepository for PostgresFeedEntryRepository {
    async fn query(
        &self,
        params: FeedEntryQueryParams,
    ) -> Result<Vec<FeedEntryDto>, RepositoryError> {
        let (cursor_published_at, cursor_id) = if let Some((published_at, id)) = params.cursor {
            (Some(published_at), Some(id))
        } else {
            (None, None)
        };

        let feed_entries = sqlx::query_file_as!(
            FeedEntryRow,
            "queries/feed_entries/find.sql",
            params.id,
            params.feed_id,
            cursor_published_at,
            cursor_id,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(feed_entries)
    }
}

struct FeedEntryRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<DbUrl>,
    feed_id: Uuid,
}

impl From<FeedEntryRow> for FeedEntryDto {
    fn from(value: FeedEntryRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            feed_id: value.feed_id,
        }
    }
}
