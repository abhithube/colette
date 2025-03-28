use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryParams, FeedEntryRepository},
};
use colette_query::IntoSelect;
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;

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

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows.iter().map(|e| FeedEntryRow(e).into()).collect())
    }
}

pub(crate) struct FeedEntryRow<'a>(pub(crate) &'a Row);

impl From<FeedEntryRow<'_>> for FeedEntry {
    fn from(FeedEntryRow(value): FeedEntryRow<'_>) -> Self {
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
