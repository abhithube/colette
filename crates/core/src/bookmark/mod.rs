pub use bookmark_repository::*;
pub use bookmark_scraper::*;
pub use bookmark_service::*;
use chrono::{DateTime, Utc};
use colette_util::base64;
use image::ImageError;
use url::Url;
use uuid::Uuid;

use crate::{
    Tag, collection,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    job, tag,
};

mod bookmark_repository;
mod bookmark_scraper;
mod bookmark_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Bookmark {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub archived_path: Option<String>,
    pub author: Option<String>,
    pub user_id: String,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
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
    Image(#[from] ImageError),

    #[error(transparent)]
    Storage(#[from] object_store::Error),

    #[error(transparent)]
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Job(#[from] job::Error),

    #[error(transparent)]
    Tag(#[from] tag::Error),

    #[error(transparent)]
    Collection(#[from] collection::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
