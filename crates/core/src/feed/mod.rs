use std::str::Utf8Error;

use chrono::{DateTime, Utc};
pub use feed_repository::*;
pub use feed_scraper::*;
pub use feed_service::*;
use url::Url;
use uuid::Uuid;

use crate::FeedEntry;

mod feed_repository;
mod feed_scraper;
mod feed_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Feed {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub link: Url,
    pub xml_url: Option<Url>,
    pub title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub entries: Option<Vec<FeedEntry>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub link: Url,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with ID: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] Utf8Error),

    #[error(transparent)]
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Database(#[from] libsql::Error),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
}
