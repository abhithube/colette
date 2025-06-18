use std::str::Utf8Error;

use chrono::{DateTime, Utc};
use colette_scraper::feed::ProcessedFeed;
pub use feed_repository::*;
pub use feed_service::*;
use url::Url;
use uuid::Uuid;

use crate::FeedEntry;

mod feed_repository;
mod feed_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Feed {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<DateTime<Utc>>,
    #[builder(default = false)]
    pub is_custom: bool,
    pub entries: Option<Vec<FeedEntry>>,
}

impl From<(Url, ProcessedFeed)> for Feed {
    fn from((source_url, value): (Url, ProcessedFeed)) -> Self {
        let feed_id = Uuid::new_v4();

        let entries = value
            .entries
            .into_iter()
            .map(|e| {
                FeedEntry::builder()
                    .link(e.link)
                    .title(e.title)
                    .published_at(e.published)
                    .maybe_description(e.description)
                    .maybe_author(e.author)
                    .maybe_thumbnail_url(e.thumbnail)
                    .feed_id(feed_id)
                    .build()
            })
            .collect();

        Feed::builder()
            .id(feed_id)
            .source_url(source_url)
            .link(value.link)
            .title(value.title)
            .maybe_description(value.description)
            .maybe_refreshed_at(value.refreshed)
            .entries(entries)
            .build()
    }
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
    Scraper(#[from] colette_scraper::feed::FeedError),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),

    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),

    #[error(transparent)]
    SqlitePool(#[from] deadpool_sqlite::PoolError),

    #[error(transparent)]
    SqliteClient(#[from] rusqlite::Error),
}
