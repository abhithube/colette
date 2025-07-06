use chrono::{DateTime, Utc};
pub use subscription_repository::*;
pub use subscription_service::*;
use uuid::Uuid;

use crate::{Feed, Tag, job, subscription_entry, tag};

mod subscription_repository;
mod subscription_service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bon::Builder)]
pub struct Subscription {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<Feed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
    pub id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("subscription not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access subscription with ID: {0}")]
    Forbidden(Uuid),

    #[error("already subscribed to feed with ID: {0}")]
    Conflict(Uuid),

    #[error(transparent)]
    Tag(#[from] tag::Error),

    #[error(transparent)]
    SubscriptionEntry(#[from] subscription_entry::Error),

    #[error(transparent)]
    Job(#[from] job::Error),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Opml(#[from] colette_opml::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),

    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),

    #[error(transparent)]
    SqlitePool(#[from] deadpool_sqlite::PoolError),

    #[error(transparent)]
    SqliteClient(#[from] rusqlite::Error),
}
