pub use archive_thumbnail_handler::*;
pub use bookmark_repository::*;
use chrono::{DateTime, Utc};
pub use create_bookmark_handler::*;
pub use delete_bookmark_handler::*;
pub use export_bookmarks_handler::*;
pub use get_bookmark_handler::*;
pub use import_bookmarks_handler::*;
pub use link_bookmark_tags_handler::*;
pub use list_bookmarks_handler::*;
pub use refresh_bookmark_handler::*;
pub use scrape_bookmark_handler::*;
pub use update_bookmark_handler::*;
use url::Url;
use uuid::Uuid;

use crate::{
    Tag,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    pagination::Cursor,
};

mod archive_thumbnail_handler;
mod bookmark_repository;
mod create_bookmark_handler;
mod delete_bookmark_handler;
mod export_bookmarks_handler;
mod get_bookmark_handler;
mod import_bookmarks_handler;
mod link_bookmark_tags_handler;
mod list_bookmarks_handler;
mod refresh_bookmark_handler;
mod scrape_bookmark_handler;
mod update_bookmark_handler;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub archived_path: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
    pub user_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveThumbnailJobData {
    pub operation: ThumbnailOperation,
    pub archived_path: Option<String>,
    pub bookmark_id: Uuid,
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
