use std::str::Utf8Error;

use chrono::{DateTime, Utc};
use colette_scraper::feed::ProcessedFeed;
pub use feed_repository::*;
pub use feed_service::*;
use url::Url;
use uuid::Uuid;

use crate::{FeedEntry, feed_entry, pagination::Cursor};

mod feed_repository;
mod feed_service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bon::Builder)]
pub struct Feed {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    #[serde(skip_serializing, default = "default_refresh_interval_min")]
    #[builder(default = 60)]
    pub refresh_interval_min: u32,
    #[serde(skip_serializing, default = "default_is_refreshing")]
    #[builder(default = false)]
    pub is_refreshing: bool,
    #[serde(skip_serializing)]
    pub refreshed_at: Option<DateTime<Utc>>,
    #[builder(default = false)]
    pub is_custom: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<FeedEntry>>,
}

fn default_refresh_interval_min() -> u32 {
    60
}
fn default_is_refreshing() -> bool {
    false
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
            .entries(entries)
            .build()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedCursor {
    pub source_url: Url,
}

impl Cursor for Feed {
    type Data = FeedCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            source_url: self.source_url.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with ID: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    FeedEntry(#[from] feed_entry::Error),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] Utf8Error),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::feed::FeedError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
