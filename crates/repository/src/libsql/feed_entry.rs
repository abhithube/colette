use chrono::DateTime;
use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryParams, FeedEntryRepository},
};
use colette_query::IntoSelect;
use libsql::Connection;
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlFeedEntryRepository {
    conn: Connection,
}

impl LibsqlFeedEntryRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for LibsqlFeedEntryRepository {
    async fn query(&self, params: FeedEntryParams) -> Result<Vec<FeedEntry>, Error> {
        let (sql, values) = params.into_select().build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut feed_entries = Vec::<FeedEntry>::new();
        while let Some(row) = rows.next().await? {
            feed_entries.push(libsql::de::from_row::<FeedEntryRow>(&row)?.into());
        }

        Ok(feed_entries)
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct FeedEntryRow {
    pub(crate) id: Uuid,
    pub(crate) link: String,
    pub(crate) title: String,
    pub(crate) published_at: i64,
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
            published_at: DateTime::from_timestamp(value.published_at, 0).unwrap(),
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            feed_id: value.feed_id,
        }
    }
}
