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
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionEntryRepository {
    pool: Pool,
}

impl SqliteSubscriptionEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for SqliteSubscriptionEntryRepository {
    async fn query(
        &self,
        params: SubscriptionEntryParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let client = self.pool.get().await?;

        let subscription_entries = client
            .interact(move |conn| {
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
                    dialect: Dialect::Sqlite,
                }
                .into_select()
                .build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<SubscriptionEntry>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(subscription_entries)
    }

    async fn save(&self, data: &SubscriptionEntry) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let Some(has_read) = data.has_read else {
            return Ok(());
        };

        let data = data.to_owned();

        client
            .interact(move |conn| {
                if has_read {
                    let (sql, values) = ReadEntryInsert {
                        subscription_id: data.subscription_id,
                        feed_entry_id: data.feed_entry_id,
                        user_id: data.user_id,
                        created_at: data.read_at.unwrap_or_else(Utc::now),
                    }
                    .into_insert()
                    .build_rusqlite(SqliteQueryBuilder);

                    conn.execute_prepared(&sql, &values)
                } else {
                    let (sql, values) = ReadEntryDelete {
                        subscription_id: data.subscription_id,
                        feed_entry_id: data.feed_entry_id,
                    }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);

                    conn.execute_prepared(&sql, &values)
                }
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for SubscriptionEntry {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            subscription_id: value.get_unwrap("subscription_id"),
            feed_entry_id: value.get_unwrap("id"),
            user_id: value.get_unwrap("user_id"),
            feed_entry: Some(SqliteRow(value).into()),
            has_read: value.get("has_read").ok(),
            read_at: value.get("created_at").ok(),
        }
    }
}
