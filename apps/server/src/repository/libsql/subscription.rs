use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Subscription, Tag,
    subscription::{Error, SubscriptionParams, SubscriptionRepository},
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
use libsql::{Connection, ffi::SQLITE_CONSTRAINT_UNIQUE};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::{LibsqlBinder, feed::FeedRow};

#[derive(Debug, Clone)]
pub struct LibsqlSubscriptionRepository {
    conn: Connection,
}

impl LibsqlSubscriptionRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for LibsqlSubscriptionRepository {
    async fn query(&self, params: SubscriptionParams) -> Result<Vec<Subscription>, Error> {
        let (sql, values) = SubscriptionSelect {
            id: params.id,
            tags: params.tags,
            user_id: params.user_id.as_deref(),
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut subscription_rows = Vec::<SubscriptionWithFeedRow>::new();
        while let Some(row) = rows.next().await? {
            subscription_rows.push(libsql::de::from_row(&row)?);
        }

        let subscription_ids = subscription_rows.iter().map(|e| e.id);

        let (sql, values) = SubscriptionTagSelect {
            subscription_ids: subscription_ids.clone(),
        }
        .into_select()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut tag_rows = Vec::<SubscriptionTagRow>::new();
        while let Some(row) = rows.next().await? {
            tag_rows.push(libsql::de::from_row(&row)?);
        }

        let (sql, values) = UnreadCountSelectMany { subscription_ids }
            .into_select()
            .build_libsql(SqliteQueryBuilder);

        #[derive(serde::Deserialize)]
        struct Row {
            subscription_id: Uuid,
            unread_count: i64,
        }

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut unread_count_rows = Vec::<Row>::new();
        while let Some(row) = rows.next().await? {
            unread_count_rows.push(libsql::de::from_row(&row)?);
        }

        let mut tag_row_map = HashMap::<Uuid, Vec<SubscriptionTagRow>>::new();
        let mut unread_count_map = HashMap::<Uuid, i64>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.subscription_id)
                .or_default()
                .push(row);
        }

        for row in unread_count_rows {
            unread_count_map
                .entry(row.subscription_id)
                .insert_entry(row.unread_count);
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
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row::<SubscriptionRow>(&row)?.into()))
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error> {
        let tx = self.conn.transaction().await?;

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
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = tx.prepare(&sql).await?;
            stmt.execute(values.into_params())
                .await
                .map_err(|e| match e {
                    libsql::Error::SqliteFailure(SQLITE_CONSTRAINT_UNIQUE, _) => {
                        Error::Conflict(data.feed_id)
                    }
                    _ => Error::Database(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = SubscriptionTagDelete {
                subscription_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = tx.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;

            let (sql, values) = SubscriptionTagInsert {
                subscription_id: data.id,
                tags: tags.iter().map(|e| SubscriptionTagById {
                    id: e.id,
                    user_id: &e.user_id,
                }),
            }
            .into_insert()
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = tx.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = SubscriptionDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: String,
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

#[derive(serde::Deserialize)]
pub struct SubscriptionTagRow {
    pub subscription_id: Uuid,
    pub id: Uuid,
    pub title: String,
    pub user_id: String,
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

#[derive(serde::Deserialize)]
pub struct SubscriptionWithFeedRow {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: String,
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
