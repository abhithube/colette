use chrono::{DateTime, Utc};
use colette_util::base64;
pub use feed_entry_repository::*;
pub use feed_entry_service::*;
use uuid::Uuid;

mod feed_entry_repository;
mod feed_entry_service;

#[derive(Clone, Debug, Default)]
pub struct FeedEntry {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: Uuid,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub id: Uuid,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed entry not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
