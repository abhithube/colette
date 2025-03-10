use chrono::{DateTime, Utc};
use colette_core::{
    SubscriptionEntry,
    common::Transaction,
    subscription_entry::{
        Error, SubscriptionEntryById, SubscriptionEntryFindByIdParams, SubscriptionEntryFindParams,
        SubscriptionEntryRepository,
    },
};
use colette_query::IntoSelect;
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};

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
    async fn find_subscription_entries(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, SubscriptionEntryRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_subscription_entry_by_id(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryFindByIdParams,
    ) -> Result<SubscriptionEntryById, Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let id = params.feed_entry_id;

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_with(&sql, values)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Database(e),
            })?;

        Ok(SubscriptionEntryById {
            feed_entry_id: row.get::<String, _>(0).parse().unwrap(),
            user_id: row.get::<String, _>(1).parse().unwrap(),
        })
    }
}

#[derive(sqlx::FromRow)]
struct SubscriptionEntryRow {
    id: String,
    link: String,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<String>,
    feed_id: String,

    subscription_id: String,
    user_id: String,
    has_read: bool,
}

impl From<SubscriptionEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryRow) -> Self {
        Self {
            entry: FeedEntryRow {
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
            subscription_id: value.subscription_id.parse().unwrap(),
            user_id: value.user_id.parse().unwrap(),
            has_read: value.has_read,
        }
    }
}
