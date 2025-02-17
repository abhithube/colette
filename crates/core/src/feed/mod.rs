use std::str::Utf8Error;

use chrono::{DateTime, Utc};
pub use feed_repository::*;
pub use feed_scraper::*;
pub use feed_service::*;
use url::Url;
use uuid::Uuid;

use crate::Tag;

mod feed_repository;
mod feed_scraper;
mod feed_service;

#[derive(Debug, Clone)]
pub struct Feed {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub xml_url: Option<Url>,
    pub folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    #[error("feed not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Conflict(ConflictError),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] Utf8Error),

    #[error(transparent)]
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    #[error("feed not cached with URL: {0}")]
    NotCached(Url),

    #[error("feed already exists with URL: {0}")]
    AlreadyExists(Url),
}
