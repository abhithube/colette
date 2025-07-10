use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryParams, FeedEntryRepository},
};
use colette_query::{IntoSelect, feed_entry::FeedEntrySelect};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteFeedEntryRepository {
    pool: Pool,
}

impl SqliteFeedEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for SqliteFeedEntryRepository {
    async fn query(&self, params: FeedEntryParams) -> Result<Vec<FeedEntry>, Error> {
        let client = self.pool.get().await?;

        let feed_entries = client
            .interact(move |conn| {
                let (sql, values) = FeedEntrySelect {
                    id: params.id,
                    feed_id: params.feed_id,
                    cursor: params.cursor,
                    limit: params.limit,
                }
                .into_select()
                .build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<FeedEntry>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(feed_entries)
    }
}

impl From<SqliteRow<'_>> for FeedEntry {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            link: value.get_unwrap::<_, String>("link").parse().unwrap(),
            title: value.get_unwrap("title"),
            published_at: value.get_unwrap("published_at"),
            description: value.get_unwrap("description"),
            author: value.get_unwrap("author"),
            thumbnail_url: value
                .get_unwrap::<_, Option<String>>("thumbnail_url")
                .and_then(|e| e.parse().ok()),
            feed_id: value.get_unwrap("feed_id"),
        }
    }
}
