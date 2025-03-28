use chrono::Utc;
use colette_core::{
    SubscriptionEntry,
    subscription_entry::{Error, SubscriptionEntryParams, SubscriptionEntryRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    read_entry::{ReadEntryDelete, ReadEntryInsert},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;
use uuid::Uuid;

use super::feed_entry::FeedEntryRow;

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

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows
            .iter()
            .map(|e| SubscriptionEntryWithFeedEntryRow(e).into())
            .collect())
    }

    async fn save(&self, data: &SubscriptionEntry) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let Some(has_read) = data.has_read else {
            return Ok(());
        };

        if has_read {
            let (sql, values) = ReadEntryInsert {
                feed_entry_id: data.entry_id,
                subscription_id: data.subscription_id,
                user_id: &data.user_id,
                created_at: Utc::now(),
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            let stmt = client.prepare_cached(&sql).await?;
            client.execute(&stmt, &values.as_params()).await?;
        } else {
            let (sql, values) = ReadEntryDelete {
                feed_entry_id: data.entry_id,
                subscription_id: data.subscription_id,
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

            let stmt = client.prepare_cached(&sql).await?;
            client.execute(&stmt, &values.as_params()).await?;
        }

        Ok(())
    }
}

struct SubscriptionEntryWithFeedEntryRow<'a>(&'a Row);

impl From<SubscriptionEntryWithFeedEntryRow<'_>> for SubscriptionEntry {
    fn from(
        SubscriptionEntryWithFeedEntryRow(value): SubscriptionEntryWithFeedEntryRow<'_>,
    ) -> Self {
        Self {
            entry_id: value.get("id"),
            subscription_id: value.get("subscription_id"),
            user_id: value.get("user_id"),
            entry: Some(FeedEntryRow(value).into()),
            has_read: value.get("has_read"),
        }
    }
}

#[derive(serde::Deserialize)]
struct SubscriptionEntryRow {
    id: Uuid,
    subscription_id: Uuid,
    user_id: String,
}

impl From<SubscriptionEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryRow) -> Self {
        Self {
            entry_id: value.id,
            subscription_id: value.subscription_id,
            user_id: value.user_id,
            entry: None,
            has_read: None,
        }
    }
}
