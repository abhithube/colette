use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository},
};
use colette_query::{IntoSelect, feed_entry::FeedEntrySelect};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteFeedEntryRepository {
    pool: Pool<Sqlite>,
}

impl SqliteFeedEntryRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for SqliteFeedEntryRepository {
    async fn find(&self, params: FeedEntryFindParams) -> Result<Vec<FeedEntry>, Error> {
        let (sql, values) = FeedEntrySelect {
            id: params.id,
            feed_id: params.feed_id,
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, FeedEntryRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct FeedEntryRow {
    pub(crate) id: Uuid,
    pub(crate) link: String,
    pub(crate) title: String,
    pub(crate) published_at: DateTime<Utc>,
    pub(crate) description: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) feed_id: Uuid,
}

impl From<FeedEntryRow> for FeedEntry {
    fn from(value: FeedEntryRow) -> Self {
        Self {
            id: value.id,
            link: value.link.parse().unwrap(),
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            feed_id: value.feed_id,
        }
    }
}
