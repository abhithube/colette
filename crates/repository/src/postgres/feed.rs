use colette_core::{
    Feed,
    feed::{Error, FeedParams, FeedRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    feed::{FeedBase, FeedInsert, FeedSelect},
    feed_entry::{FeedEntryInsert, FeedEntryInsertBatch},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;

use super::{IdRow, PgRow, PreparedClient as _};

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

        let (sql, values) = FeedSelect {
            id: params.id,
            source_urls: params
                .source_urls
                .as_ref()
                .map(|e| e.iter().map(|e| e.as_str()).collect()),
            cursor: params.cursor.as_deref(),
            limit: params.limit,
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let feeds = client.query_prepared::<Feed>(&sql, &values).await?;

        Ok(feeds)
    }

    async fn save(&self, data: &mut Feed) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        data.id = {
            let feed = FeedInsert {
                feeds: [FeedBase {
                    id: data.id,
                    source_url: data.source_url.as_str(),
                    link: data.link.as_str(),
                    title: &data.title,
                    description: data.description.as_deref(),
                    refreshed_at: data.refreshed_at,
                    is_custom: data.is_custom,
                }],
                upsert: true,
            };

            let (sql, values) = feed.into_insert().build_postgres(PostgresQueryBuilder);
            let row = tx.query_one_prepared::<IdRow>(&sql, &values).await?;

            row.id
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
                feed_id: data.id,
            });

            let (sql, values) = FeedEntryInsertBatch(entries)
                .into_insert()
                .build_postgres(PostgresQueryBuilder);

            tx.execute_prepared(&sql, &values).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Feed {
    fn from(PgRow(value): PgRow<'_>) -> Self {
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
