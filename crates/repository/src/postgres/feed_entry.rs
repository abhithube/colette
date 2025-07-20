use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryParams, FeedEntryRepository},
};
use colette_query::{IntoSelect, feed_entry::FeedEntrySelect};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

use crate::postgres::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: PgPool,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for PostgresFeedEntryRepository {
    async fn query(&self, params: FeedEntryParams) -> Result<Vec<FeedEntry>, Error> {
        let (sql, values) = FeedEntrySelect {
            id: params.id,
            feed_id: params.feed_id,
            cursor: params.cursor,
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, FeedEntryRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, sqlx::Type, sqlx::FromRow)]
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
            feed_id: value.feed_id,
        }
    }
}
