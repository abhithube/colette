use chrono::{DateTime, Utc};
pub use feed_entry_repository::*;
pub use get_feed_entry_handler::*;
pub use list_feed_entries_handler::*;
use url::Url;
use uuid::Uuid;

use crate::pagination::Cursor;

mod feed_entry_repository;
mod get_feed_entry_handler;
mod list_feed_entries_handler;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedEntry {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub feed_id: Uuid,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedEntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor for FeedEntry {
    type Data = FeedEntryCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            published_at: self.published_at,
            id: self.id,
        }
    }
}
