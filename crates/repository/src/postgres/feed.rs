use chrono::{DateTime, Utc};
use colette_core::{
    Feed,
    feed::{Error, FeedParams, FeedRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    feed::{FeedBase, FeedInsert, FeedSelect},
    feed_entry::{FeedEntryInsert, FeedEntryInsertBatch},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

use crate::postgres::{DbUrl, IdRow};

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: PgPool,
}

impl PostgresFeedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn query(&self, params: FeedParams) -> Result<Vec<Feed>, Error> {
        let (sql, values) = FeedSelect {
            id: params.id,
            source_urls: params
                .source_urls
                .as_ref()
                .map(|e| e.iter().map(|e| e.as_str()).collect()),
            ready_to_refresh: params.ready_to_refresh,
            cursor: params.cursor.as_ref().map(|e| e.as_str()),
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &mut Feed) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        data.id = {
            let feed = FeedInsert {
                feeds: [FeedBase {
                    id: data.id,
                    source_url: data.source_url.as_str(),
                    link: data.link.as_str(),
                    title: &data.title,
                    description: data.description.as_deref(),
                    refresh_interval_min: data.refresh_interval_min as i32,
                    is_refreshing: data.is_refreshing,
                    refreshed_at: data.refreshed_at,
                    is_custom: data.is_custom,
                }],
                upsert: true,
            };

            let (sql, values) = feed.into_insert().build_sqlx(PostgresQueryBuilder);
            let row = sqlx::query_as_with::<_, IdRow, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await?;

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
                .build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::Type, sqlx::FromRow)]
pub(crate) struct FeedRow {
    pub(crate) id: Uuid,
    pub(crate) source_url: DbUrl,
    link: DbUrl,
    title: String,
    description: Option<String>,
    refresh_interval_min: i32,
    is_refreshing: bool,
    refreshed_at: Option<DateTime<Utc>>,
    is_custom: bool,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            source_url: value.source_url.0,
            link: value.link.0,
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at,
            refresh_interval_min: value.refresh_interval_min as u32,
            is_refreshing: value.is_refreshing,
            is_custom: value.is_custom,
            entries: None,
        }
    }
}
