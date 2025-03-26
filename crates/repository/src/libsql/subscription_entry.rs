use chrono::Utc;
use colette_core::{
    SubscriptionEntry,
    subscription_entry::{Error, SubscriptionEntryParams, SubscriptionEntryRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    read_entry::{ReadEntryDelete, ReadEntryInsert},
};
use libsql::Connection;
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::{LibsqlBinder, feed_entry::FeedEntryRow};

#[derive(Debug, Clone)]
pub struct LibsqlSubscriptionEntryRepository {
    conn: Connection,
}

impl LibsqlSubscriptionEntryRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for LibsqlSubscriptionEntryRepository {
    async fn query(
        &self,
        params: SubscriptionEntryParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let (sql, values) = params.into_select().build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut subscription_entries = Vec::<SubscriptionEntry>::new();
        while let Some(row) = rows.next().await? {
            subscription_entries
                .push(libsql::de::from_row::<SubscriptionEntryWithFeedEntryRow>(&row)?.into());
        }

        Ok(subscription_entries)
    }

    async fn save(&self, data: &SubscriptionEntry) -> Result<(), Error> {
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
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = self.conn.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;
        } else {
            let (sql, values) = ReadEntryDelete {
                feed_entry_id: data.entry_id,
                subscription_id: data.subscription_id,
            }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = self.conn.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct SubscriptionEntryWithFeedEntryRow {
    id: Uuid,
    subscription_id: Uuid,
    user_id: String,
    has_read: bool,

    link: String,
    title: String,
    published_at: i64,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<String>,
    feed_id: Uuid,
}

impl From<SubscriptionEntryWithFeedEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryWithFeedEntryRow) -> Self {
        Self {
            entry_id: value.id,
            subscription_id: value.subscription_id,
            user_id: value.user_id,
            entry: Some(
                FeedEntryRow {
                    id: value.id,
                    link: value.link,
                    title: value.title,
                    published_at: value.published_at,
                    description: value.description,
                    author: value.author,
                    thumbnail_url: value.thumbnail_url,
                    feed_id: value.feed_id,
                }
                .into(),
            ),
            has_read: Some(value.has_read),
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
