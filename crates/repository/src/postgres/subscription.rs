use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Subscription, Tag,
    subscription::{Error, SubscriptionParams, SubscriptionRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed_entry::UnreadCountSelectMany,
    subscription::{SubscriptionDelete, SubscriptionInsert},
    subscription_tag::{SubscriptionTagDelete, SubscriptionTagInsert, SubscriptionTagSelect},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{Row, error::SqlState};
use uuid::Uuid;

use super::feed::FeedRow;

#[derive(Debug, Clone)]
pub struct PostgresSubscriptionRepository {
    pool: Pool,
}

impl PostgresSubscriptionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn query(&self, params: SubscriptionParams) -> Result<Vec<Subscription>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut subscriptions = rows
            .iter()
            .map(|e| Subscription::from(SubscriptionWithFeedRow(e)))
            .collect::<Vec<_>>();

        let subscription_ids = subscriptions.iter().map(|e| e.id);

        let (sql, values) = SubscriptionTagSelect {
            subscription_ids: subscription_ids.clone(),
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        let mut tag_row_map = HashMap::<Uuid, Vec<SubscriptionTagRow>>::new();

        let tag_rows = rows
            .iter()
            .map(SubscriptionTagRow::from)
            .collect::<Vec<_>>();
        for row in tag_rows {
            tag_row_map
                .entry(row.subscription_id)
                .or_default()
                .push(row);
        }

        let (sql, values) = UnreadCountSelectMany { subscription_ids }
            .into_select()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        let mut unread_count_map = HashMap::<Uuid, i64>::new();

        for row in rows {
            unread_count_map
                .entry(row.get("id"))
                .insert_entry(row.get("unread_count"));
        }

        for subscription in subscriptions.iter_mut() {
            subscription.tags = tag_row_map
                .remove(&subscription.id)
                .map(|e| e.into_iter().map(Into::into).collect());
            subscription.unread_count = unread_count_map.remove(&subscription.id);
        }

        Ok(subscriptions)
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error> {
        let mut client = self.pool.get().await?;

        let tx = client.transaction().await?;

        {
            let (sql, values) = SubscriptionInsert {
                id: data.id,
                title: &data.title,
                feed_id: data.feed_id,
                user_id: &data.user_id,
                created_at: data.created_at,
                updated_at: data.updated_at,
                upsert: false,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.feed_id),
                    _ => Error::Database(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = SubscriptionTagDelete {
                subscription_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params()).await?;

            let (sql, values) = SubscriptionTagInsert {
                subscription_id: data.id,
                user_id: &data.user_id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params()).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = SubscriptionDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }
}

struct SubscriptionTagRow {
    subscription_id: Uuid,
    id: Uuid,
    title: String,
    user_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<&Row> for SubscriptionTagRow {
    fn from(value: &Row) -> Self {
        Self {
            subscription_id: value.get("subscription_id"),
            id: value.get("id"),
            title: value.get("title"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}

impl From<SubscriptionTagRow> for Tag {
    fn from(value: SubscriptionTagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

struct SubscriptionWithFeedRow<'a>(&'a Row);

impl From<SubscriptionWithFeedRow<'_>> for Subscription {
    fn from(SubscriptionWithFeedRow(value): SubscriptionWithFeedRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            title: value.get("title"),
            feed_id: value.get("feed_id"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            feed: if value.len() > 6 {
                Some(FeedRow(value).into())
            } else {
                None
            },
            tags: None,
            unread_count: None,
        }
    }
}
