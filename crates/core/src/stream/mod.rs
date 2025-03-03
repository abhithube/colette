use chrono::{DateTime, Utc};
use colette_util::base64;
use sea_orm::DbErr;
pub use stream_repository::*;
pub use stream_service::*;
use uuid::Uuid;

use crate::feed_entry::FeedEntryFilter;

mod stream_repository;
mod stream_service;

#[derive(Debug, Clone)]
pub struct Stream {
    pub id: Uuid,
    pub title: String,
    pub filter: FeedEntryFilter,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stream not found with id: {0}")]
    NotFound(Uuid),

    #[error("stream already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] DbErr),
}
