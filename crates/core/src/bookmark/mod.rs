use apalis_redis::RedisError;
pub use bookmark_repository::*;
pub use bookmark_scraper::*;
pub use bookmark_service::*;
use chrono::{DateTime, Utc};
use colette_util::base64;
use image::ImageError;
use sea_orm::DbErr;
use url::Url;
use uuid::Uuid;

use crate::Tag;

mod bookmark_repository;
mod bookmark_scraper;
mod bookmark_service;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub archived_path: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bookmark not found with id: {0}")]
    NotFound(Uuid),

    #[error("bookmark already exists with URL: {0}")]
    Conflict(Url),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Image(#[from] ImageError),

    #[error(transparent)]
    Storage(#[from] object_store::Error),

    #[error(transparent)]
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] DbErr),

    #[error(transparent)]
    Redis(#[from] RedisError),
}
