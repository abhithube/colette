use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryParams, FeedEntryRepository},
};
use colette_query::{IntoSelect, feed_entry::FeedEntrySelect};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: Pool,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for PostgresFeedEntryRepository {
    async fn query(&self, params: FeedEntryParams) -> Result<Vec<FeedEntry>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = FeedEntrySelect {
            id: params.id,
            feed_id: params.feed_id,
            cursor: params.cursor,
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let feed_entries = client.query_prepared::<FeedEntry>(&sql, &values).await?;

        Ok(feed_entries)
    }
}

impl From<PgRow<'_>> for FeedEntry {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            link: value.get::<_, String>("link").parse().unwrap(),
            title: value.get("title"),
            published_at: value.get("published_at"),
            description: value.get("description"),
            author: value.get("author"),
            thumbnail_url: value
                .get::<_, Option<String>>("thumbnail_url")
                .and_then(|e| e.parse().ok()),
            feed_id: value.get("feed_id"),
        }
    }
}
