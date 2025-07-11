use chrono::Utc;
use colette_core::{
    SubscriptionEntry,
    subscription_entry::{Error, SubscriptionEntryParams, SubscriptionEntryRepository},
};
use colette_query::{
    Dialect, IntoDelete, IntoInsert, IntoSelect,
    feed_entry::SubscriptionEntrySelect,
    read_entry::{ReadEntryDelete, ReadEntryInsert},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresSubscriptionEntryRepository {
    pool: Pool,
}

impl PostgresSubscriptionEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for PostgresSubscriptionEntryRepository {
    async fn query(
        &self,
        params: SubscriptionEntryParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let client = self.pool.get().await?;

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
            dialect: Dialect::Postgres,
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let subscription_entries = client
            .query_prepared::<SubscriptionEntry>(&sql, &values)
            .await?;

        Ok(subscription_entries)
    }

    async fn save(&self, data: &SubscriptionEntry) -> Result<(), Error> {
        let client = self.pool.get().await?;

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
            .build_postgres(PostgresQueryBuilder);

            client.execute_prepared(&sql, &values).await?;
        } else {
            let (sql, values) = ReadEntryDelete {
                subscription_id: data.subscription_id,
                feed_entry_id: data.feed_entry_id,
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

            client.execute_prepared(&sql, &values).await?;
        }

        Ok(())
    }
}

impl From<PgRow<'_>> for SubscriptionEntry {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            subscription_id: value.get("subscription_id"),
            feed_entry_id: value.get("id"),
            user_id: value.get("user_id"),
            feed_entry: Some(PgRow(value).into()),
            has_read: value.try_get("has_read").ok(),
            read_at: value.try_get("created_at").ok(),
        }
    }
}
