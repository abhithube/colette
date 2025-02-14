pub use bookmark_repository::*;
pub use bookmark_service::*;
use chrono::{DateTime, Utc};
use colette_util::base64;
use uuid::Uuid;

use crate::Tag;

mod bookmark_repository;
mod bookmark_service;

#[derive(Clone, Debug, Default)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub archived_url: Option<String>,
    pub author: Option<String>,
    pub folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bookmark not found with id: {0}")]
    NotFound(Uuid),

    #[error("bookmark already exists with URL: {0}")]
    Conflict(String),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Archiver(#[from] colette_archiver::Error),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
