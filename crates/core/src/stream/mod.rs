use chrono::{DateTime, Utc};
use colette_util::base64;
pub use stream_repository::*;
pub use stream_service::*;
use uuid::Uuid;

use crate::subscription_entry::SubscriptionEntryFilter;

mod stream_repository;
mod stream_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Stream {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub filter: SubscriptionEntryFilter,
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stream not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access stream with ID: {0}")]
    Forbidden(Uuid),

    #[error("stream already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
