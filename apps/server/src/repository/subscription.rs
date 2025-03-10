use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Subscription, Tag,
    common::Transaction,
    subscription::{
        Error, SubscriptionById, SubscriptionCreateParams, SubscriptionDeleteParams,
        SubscriptionEntryUpdateParams, SubscriptionFindByIdParams, SubscriptionFindParams,
        SubscriptionRepository, SubscriptionTagsLinkParams, SubscriptionUpdateParams,
    },
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    feed_entry::UnreadCountSelectMany,
    subscription_tag::{
        SubscriptionTagDeleteMany, SubscriptionTagSelectMany, SubscriptionTagUpsertMany,
    },
};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};

use super::feed::FeedRow;

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionRepository {
    pool: Pool<Sqlite>,
}

impl SqliteSubscriptionRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for SqliteSubscriptionRepository {
    async fn find_subscriptions(
        &self,
        params: SubscriptionFindParams,
    ) -> Result<Vec<Subscription>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let subscription_rows = sqlx::query_as_with::<_, SubscriptionRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let subscription_ids = subscription_rows.iter().map(|e| e.id.to_string());

        let (sql, values) = SubscriptionTagSelectMany {
            subscription_ids: subscription_ids.clone(),
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let tag_rows = sqlx::query_as_with::<_, SubscriptionTagRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let (sql, values) = UnreadCountSelectMany { subscription_ids }
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let unread_count_rows = sqlx::query_with(&sql, values).fetch_all(&self.pool).await?;

        let mut tag_row_map = HashMap::<String, Vec<SubscriptionTagRow>>::new();
        let mut unread_count_map = HashMap::<String, i64>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.subscription_id.clone())
                .or_default()
                .push(row);
        }

        for row in unread_count_rows {
            unread_count_map.entry(row.get(0)).insert_entry(row.get(1));
        }

        let subscriptions = subscription_rows
            .into_iter()
            .map(|subscription| {
                SubscriptionWithTagsAndCount {
                    tags: tag_row_map.remove(&subscription.id),
                    unread_count: unread_count_map.remove(&subscription.id),
                    subscription,
                }
                .into()
            })
            .collect();

        Ok(subscriptions)
    }

    async fn find_subscription_by_id(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionFindByIdParams,
    ) -> Result<SubscriptionById, Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let id = params.id;

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_with(&sql, values)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Database(e),
            })?;

        Ok(SubscriptionById {
            id: row.get::<String, _>(0).parse().unwrap(),
            user_id: row.get::<String, _>(1).parse().unwrap(),
        })
    }

    async fn create_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionCreateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let feed_id = params.feed_id;

        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(tx.as_mut())
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(feed_id),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionUpdateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        if params.title.is_none() {
            return Ok(());
        }

        let (sql, values) = params.into_update().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }

    async fn delete_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionDeleteParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let (sql, values) = params.into_delete().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }

    async fn link_tags(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionTagsLinkParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;
        let conn = tx.as_mut();

        let (sql, values) = SubscriptionTagDeleteMany {
            subscription_id: params.subscription_id,
            tag_ids: params.tags.iter().map(|e| e.id.to_string()),
        }
        .into_delete()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;

        let (sql, values) = SubscriptionTagUpsertMany {
            subscription_id: params.subscription_id,
            tags: params.tags,
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(conn).await?;

        Ok(())
    }

    async fn update_subscription_entry(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryUpdateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        if params.has_read {
            let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;
        } else {
            let (sql, values) = params.into_delete().build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct SubscriptionRow {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub feed_id: String,
    pub link: String,
    pub xml_url: Option<String>,
    pub feed_title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
pub struct SubscriptionTagRow {
    pub subscription_id: String,
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SubscriptionTagRow> for Tag {
    fn from(value: SubscriptionTagRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: value.created_at,
            updated_at: value.updated_at,
            ..Default::default()
        }
    }
}

pub struct SubscriptionWithTagsAndCount {
    pub subscription: SubscriptionRow,
    pub tags: Option<Vec<SubscriptionTagRow>>,
    pub unread_count: Option<i64>,
}

impl From<SubscriptionWithTagsAndCount> for Subscription {
    fn from(value: SubscriptionWithTagsAndCount) -> Self {
        Self {
            id: value.subscription.id.parse().unwrap(),
            title: value.subscription.title,
            user_id: value.subscription.user_id.parse().unwrap(),
            created_at: value.subscription.created_at,
            updated_at: value.subscription.updated_at,
            feed: FeedRow {
                id: value.subscription.feed_id,
                link: value.subscription.link,
                xml_url: value.subscription.xml_url,
                title: value.subscription.feed_title,
                description: value.subscription.description,
                refreshed_at: value.subscription.refreshed_at,
            }
            .into(),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
            unread_count: value.unread_count,
        }
    }
}
