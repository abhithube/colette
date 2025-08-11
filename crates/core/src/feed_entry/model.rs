use std::fmt;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{feed::FeedId, pagination::Cursor};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedEntry {
    pub id: FeedEntryId,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub feed_id: FeedId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FeedEntryId(Uuid);

impl FeedEntryId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for FeedEntryId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for FeedEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedEntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: FeedEntryId,
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
