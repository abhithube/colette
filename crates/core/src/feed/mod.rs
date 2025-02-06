pub use feed_repository::*;
pub use feed_scraper::*;
pub use feed_service::*;
use uuid::Uuid;

use crate::Tag;

mod feed_repository;
mod feed_scraper;
mod feed_service;

#[derive(Clone, Debug, Default)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub xml_url: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
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
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    #[error("feed not cached with URL: {0}")]
    NotCached(String),

    #[error("feed already exists with URL: {0}")]
    AlreadyExists(String),
}
