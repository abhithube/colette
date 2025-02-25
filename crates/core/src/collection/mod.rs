use chrono::{DateTime, Utc};
use colette_util::base64;
pub use collection_repository::*;
pub use collection_service::*;
use uuid::Uuid;

use crate::filter::{BooleanOp, DateOp, NumberOp, TextOp};

mod collection_repository;
mod collection_service;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with id: {0}")]
    NotFound(Uuid),

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
