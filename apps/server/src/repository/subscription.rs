use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Subscription, Tag,
    subscription::{Error, SubscriptionFindParams, SubscriptionRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed_entry::UnreadCountSelectMany,
    subscription::{
        SubscriptionDelete, SubscriptionInsert, SubscriptionSelect, SubscriptionSelectOne,
    },
    subscription_tag::{
        SubscriptionTagById, SubscriptionTagDelete, SubscriptionTagInsert, SubscriptionTagSelect,
    },
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

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
    async fn find(&self, params: SubscriptionFindParams) -> Result<Vec<Subscription>, Error> {
        let (sql, values) = SubscriptionSelect {
            id: params.id,
            tags: params.tags,
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let subscription_rows = sqlx::query_as_with::<_, SubscriptionWithFeedRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let subscription_ids = subscription_rows.iter().map(|e| e.id);

        let (sql, values) = SubscriptionTagSelect {
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

        let mut tag_row_map = HashMap::<Uuid, Vec<SubscriptionTagRow>>::new();
        let mut unread_count_map = HashMap::<Uuid, i64>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.subscription_id)
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

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscription>, Error> {
        let (sql, values) = SubscriptionSelectOne { id }
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, SubscriptionRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, data: &Subscription, upsert: bool) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = SubscriptionInsert {
            id: data.id,
            title: &data.title,
            feed_id: data.feed_id,
            user_id: data.user_id,
            upsert,
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        if let Some(ref tags) = data.tags {
            let (sql, values) = SubscriptionTagDelete {
                subscription_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

            let (sql, values) = SubscriptionTagInsert {
                subscription_id: data.id,
                tags: tags.iter().map(|e| SubscriptionTagById {
                    id: e.id,
                    user_id: e.user_id,
                }),
            }
            .into_insert()
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
        }

        sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(data.feed_id)
                }
                _ => Error::Database(e),
            })?;

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = SubscriptionDelete { id }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SubscriptionRow> for Subscription {
    fn from(value: SubscriptionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            feed_id: value.feed_id,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            feed: None,
            tags: None,
            unread_count: None,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct SubscriptionTagRow {
    pub subscription_id: Uuid,
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SubscriptionTagRow> for Tag {
    fn from(value: SubscriptionTagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            feed_count: None,
            bookmark_count: None,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct SubscriptionWithFeedRow {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub link: Option<String>,
    pub xml_url: Option<String>,
    pub feed_title: Option<String>,
    pub description: Option<String>,
    pub refreshed_at: Option<DateTime<Utc>>,
}

pub struct SubscriptionWithTagsAndCount {
    pub subscription: SubscriptionWithFeedRow,
    pub tags: Option<Vec<SubscriptionTagRow>>,
    pub unread_count: Option<i64>,
}

impl From<SubscriptionWithTagsAndCount> for Subscription {
    fn from(value: SubscriptionWithTagsAndCount) -> Self {
        Self {
            id: value.subscription.id,
            title: value.subscription.title,
            feed_id: value.subscription.feed_id,
            user_id: value.subscription.user_id,
            created_at: value.subscription.created_at,
            updated_at: value.subscription.updated_at,
            feed: Some(
                FeedRow {
                    id: value.subscription.feed_id,
                    link: value.subscription.link.unwrap(),
                    xml_url: value.subscription.xml_url,
                    title: value.subscription.feed_title.unwrap(),
                    description: value.subscription.description,
                    refreshed_at: value.subscription.refreshed_at,
                }
                .into(),
            ),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
            unread_count: value.unread_count,
        }
    }
}
