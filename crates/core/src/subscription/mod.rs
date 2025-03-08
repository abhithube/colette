use chrono::{DateTime, Utc};
use sea_orm::DbErr;
pub use subscription_repository::*;
pub use subscription_service::*;
use uuid::Uuid;

use crate::{Feed, Tag, subscription_entry, tag};

mod subscription_repository;
mod subscription_service;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub feed: Feed,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub id: Uuid,
    pub title: String,
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
    Database(#[from] DbErr),
}
