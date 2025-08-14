use std::fmt;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    Tag,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    pagination::Cursor,
    auth::UserId,
};

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: BookmarkId,
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub archived_path: Option<String>,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Option<Vec<Tag>>,
}

impl Bookmark {
    pub fn authorize(&self, user_id: UserId) -> Result<(), BookmarkError> {
        if self.user_id != user_id {
            return Err(BookmarkError::Forbidden(user_id));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BookmarkId(Uuid);

impl BookmarkId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for BookmarkId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for BookmarkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BookmarkCursor {
    pub created_at: DateTime<Utc>,
}

impl Cursor for Bookmark {
    type Data = BookmarkCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeBookmarkJobData {
    pub url: Url,
    pub user_id: UserId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJobData {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
    pub bookmark_id: BookmarkId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ThumbnailOperation {
    Upload(Url),
    Delete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkFilter {
    Text {
        field: BookmarkTextField,
        op: TextOp,
    },
    Number {
        field: BookmarkNumberField,
        op: NumberOp,
    },
    Boolean {
        field: BookmarkBooleanField,
        op: BooleanOp,
    },
    Date {
        field: BookmarkDateField,
        op: DateOp,
    },

    And(Vec<BookmarkFilter>),
    Or(Vec<BookmarkFilter>),
    Not(Box<BookmarkFilter>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkTextField {
    Link,
    Title,
    Author,
    Tag,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkNumberField {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkBooleanField {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkDateField {
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkError {
    #[error("not authorized to access bookmark with ID: {0}")]
    Forbidden(UserId),
}
