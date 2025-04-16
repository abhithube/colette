use colette_core::{
    Feed,
    feed::{Error, FeedParams, FeedRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    feed::FeedInsert,
    feed_entry::{FeedEntryInsert, FeedEntryInsertBatch},
};
use deadpool_postgres::Pool;
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;
use url::Url;

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: Pool,
}

impl PostgresFeedRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn query(&self, params: FeedParams) -> Result<Vec<Feed>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows.iter().map(|e| FeedRow(e).into()).collect())
    }

    async fn save(&self, data: &Feed) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        let feed_id = {
            let feed = FeedInsert {
                id: data.id,
                source_url: data.source_url.as_str(),
                link: data.link.as_str(),
                title: &data.title,
                description: data.description.as_deref(),
                refreshed_at: data.refreshed_at,
                is_custom: data.is_custom,
            };

            let (sql, values) = feed.into_insert().build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            let row = tx.query_one(&stmt, &values.as_params()).await?;

            row.get("id")
        };

        if let Some(ref entries) = data.entries {
            let entries = entries.iter().map(|e| FeedEntryInsert {
                id: e.id,
                link: e.link.as_str(),
                title: &e.title,
                published_at: e.published_at,
                description: e.description.as_deref(),
                author: e.author.as_deref(),
                thumbnail_url: e.thumbnail_url.as_ref().map(|e| e.as_str()),
                feed_id,
            });

            let (sql, values) = FeedEntryInsertBatch(entries)
                .into_insert()
                .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params()).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = FeedParams::default()
            .into_select()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        let urls = rows
            .iter()
            .map(|e| Ok(e.get::<_, String>("source_url").parse().unwrap()))
            .collect::<Vec<_>>();

        Ok(stream::iter(urls).boxed())
    }
}

struct FeedRow<'a>(&'a Row);

impl From<FeedRow<'_>> for Feed {
    fn from(FeedRow(value): FeedRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            source_url: value.get::<_, String>("source_url").parse().unwrap(),
            link: value.get::<_, String>("link").parse().unwrap(),
            title: value.get("title"),
            description: value.get("description"),
            refreshed_at: value.get("refreshed_at"),
            is_custom: value.get("is_custom"),
            entries: None,
        }
    }
}
