use chrono::{DateTime, Utc};
pub use subscription_repository::*;
pub use subscription_service::*;
use uuid::Uuid;

use crate::{Feed, Tag, job, subscription_entry, tag};

mod subscription_repository;
mod subscription_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Subscription {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: String,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    pub feed: Option<Feed>,
    pub tags: Option<Vec<Tag>>,
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
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
}
