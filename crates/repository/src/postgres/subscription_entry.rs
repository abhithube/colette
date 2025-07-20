use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry, SubscriptionEntry,
    subscription_entry::{Error, SubscriptionEntryParams, SubscriptionEntryRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed_entry::SubscriptionEntrySelect,
    read_entry::{ReadEntryDelete, ReadEntryInsert},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

use crate::postgres::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresSubscriptionEntryRepository {
    pool: PgPool,
}

impl PostgresSubscriptionEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for PostgresSubscriptionEntryRepository {
    async fn query(
        &self,
        params: SubscriptionEntryParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let (sql, values) = SubscriptionEntrySelect {
            filter: params.filter,
            subscription_id: params.subscription_id,
            feed_entry_id: params.feed_entry_id,
            has_read: params.has_read,
            tags: params.tags,
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit.map(|e| e as u64),
            with_read_entry: params.with_read_entry,
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, SubscriptionEntryRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &SubscriptionEntry) -> Result<(), Error> {
        let Some(has_read) = data.has_read else {
            return Ok(());
        };

        if has_read {
            let (sql, values) = ReadEntryInsert {
                subscription_id: data.subscription_id,
                feed_entry_id: data.feed_entry_id,
                user_id: data.user_id,
                created_at: data.read_at.unwrap_or_else(Utc::now),
            }
            .into_insert()
            .build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&self.pool).await?;
        } else {
            let (sql, values) = ReadEntryDelete {
                subscription_id: data.subscription_id,
                feed_entry_id: data.feed_entry_id,
            }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&self.pool).await?;
        }

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SubscriptionEntryRow {
    id: Uuid,
    subscription_id: Uuid,
    user_id: Uuid,
    has_read: Option<bool>,
    created_at: Option<DateTime<Utc>>,

    #[sqlx(default)]
    link: Option<DbUrl>,
    #[sqlx(default)]
    title: Option<String>,
    #[sqlx(default)]
    published_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    description: Option<String>,
    #[sqlx(default)]
    author: Option<String>,
    #[sqlx(default)]
    thumbnail_url: Option<DbUrl>,
    #[sqlx(default)]
    feed_id: Option<Uuid>,
}

impl From<SubscriptionEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryRow) -> Self {
        Self {
            subscription_id: value.subscription_id,
            feed_entry_id: value.id,
            user_id: value.user_id,
            feed_entry: if let Some(feed_id) = value.feed_id
                && let Some(link) = value.link.map(|e| e.0)
                && let Some(title) = value.title
                && let Some(published_at) = value.published_at
            {
                Some(FeedEntry {
                    id: value.id,
                    link,
                    title,
                    published_at,
                    description: value.description,
                    thumbnail_url: value.thumbnail_url.map(|e| e.0),
                    author: value.author,
                    feed_id,
                })
            } else {
                None
            },
            has_read: value.has_read,
            read_at: value.created_at,
        }
    }
}
