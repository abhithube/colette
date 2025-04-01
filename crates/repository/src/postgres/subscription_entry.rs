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
                subscription_id: data.subscription_id,
                feed_entry_id: data.feed_entry_id,
                user_id: &data.user_id,
                created_at: data.read_at.unwrap_or_else(Utc::now),
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            let stmt = client.prepare_cached(&sql).await?;
            client.execute(&stmt, &values.as_params()).await?;
        } else {
            let (sql, values) = ReadEntryDelete {
                subscription_id: data.subscription_id,
                feed_entry_id: data.feed_entry_id,
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
            subscription_id: value.get("subscription_id"),
            feed_entry_id: value.get("id"),
            user_id: value.get("user_id"),
            feed_entry: Some(FeedEntryRow(value).into()),
            has_read: value.try_get("has_read").ok(),
            read_at: value.try_get("created_at").ok(),
        }
    }
}
