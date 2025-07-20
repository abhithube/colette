pub use bookmark_repository::*;
pub use bookmark_service::*;
use chrono::{DateTime, Utc};
use image::ImageError;
use url::Url;
use uuid::Uuid;

use crate::{
    Tag, collection,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    job,
    pagination::Cursor,
    tag,
};

mod bookmark_repository;
mod bookmark_service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bon::Builder)]
pub struct Bookmark {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub archived_path: Option<String>,
    pub author: Option<String>,
    #[serde(skip_serializing, default = "Uuid::new_v4")]
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
pub enum Error {
    #[error("bookmark not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access bookmark with ID: {0}")]
    Forbidden(Uuid),

    #[error("bookmark already exists with URL: {0}")]
    Conflict(Url),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Tag(#[from] tag::Error),

    #[error(transparent)]
    Collection(#[from] collection::Error),

    #[error(transparent)]
    Job(#[from] job::Error),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Netscape(#[from] colette_netscape::Error),

    #[error(transparent)]
    Image(#[from] ImageError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::bookmark::BookmarkError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),
}
