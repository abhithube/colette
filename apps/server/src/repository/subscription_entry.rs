use chrono::{DateTime, Utc};
use colette_core::{
    SubscriptionEntry,
    subscription_entry::{Error, SubscriptionEntryFindParams, SubscriptionEntryRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed_entry::{SubscriptionEntrySelect, SubscriptionEntrySelectOne},
    read_entry::{ReadEntryDelete, ReadEntryInsert},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use super::feed_entry::FeedEntryRow;

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionEntryRepository {
    pool: Pool<Sqlite>,
}

impl SqliteSubscriptionEntryRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for SqliteSubscriptionEntryRepository {
    async fn find(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let (sql, values) = SubscriptionEntrySelect {
            filter: params.filter,
            id: params.id,
            subscription_id: params.subscription_id,
            has_read: params.has_read,
            tags: params.tags,
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, SubscriptionEntryWithFeedEntryRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(
        &self,
        feed_entry_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<Option<SubscriptionEntry>, Error> {
        let (sql, values) = SubscriptionEntrySelectOne {
            feed_entry_id,
            subscription_id,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, SubscriptionEntryRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
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
            }
            .into_insert()
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&self.pool).await?;
        } else {
            let (sql, values) = ReadEntryDelete {
                feed_entry_id: data.entry_id,
                subscription_id: data.subscription_id,
            }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&self.pool).await?;
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SubscriptionEntryWithFeedEntryRow {
    id: Uuid,
    subscription_id: Uuid,
    user_id: String,
    has_read: bool,

    link: String,
    title: String,
    published_at: DateTime<Utc>,
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

#[derive(sqlx::FromRow)]
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
