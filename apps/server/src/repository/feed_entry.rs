use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository},
};
use colette_query::IntoSelect;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};

use super::common::parse_timestamp;

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
    async fn find_feed_entries(
        &self,
        params: FeedEntryFindParams,
    ) -> Result<Vec<FeedEntry>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, FeedEntryRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct FeedEntryRow {
    pub(crate) id: String,
    pub(crate) link: String,
    pub(crate) title: String,
    pub(crate) published_at: i32,
    pub(crate) description: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) feed_id: String,
}

impl From<FeedEntryRow> for FeedEntry {
    fn from(value: FeedEntryRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            link: value.link.parse().unwrap(),
            title: value.title,
            published_at: parse_timestamp(value.published_at).unwrap(),
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            feed_id: value.feed_id.parse().unwrap(),
        }
    }
}
