use chrono::{DateTime, Utc};
use colette_util::base64;
pub use stream_repository::*;
pub use stream_service::*;
use uuid::Uuid;

use crate::filter::{BooleanOp, DateOp, NumberOp, TextOp};

mod stream_repository;
mod stream_service;

#[derive(Debug, Clone)]
pub struct Stream {
    pub id: Uuid,
    pub title: String,
    pub filter: FeedEntryFilter,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryFilter {
    Text {
        field: FeedEntryTextField,
        op: TextOp,
    },
    Number {
        field: FeedEntryNumberField,
        op: NumberOp,
    },
    Boolean {
        field: FeedEntryBooleanField,
        op: BooleanOp,
    },
    Date {
        field: FeedEntryDateField,
        op: DateOp,
    },

    And(Vec<FeedEntryFilter>),
    Or(Vec<FeedEntryFilter>),
    Not(Box<FeedEntryFilter>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryTextField {
    Link,
    Title,
    Description,
    Author,
    Tag,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryNumberField {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryBooleanField {
    HasRead,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryDateField {
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
    #[error("stream not found with id: {0}")]
    NotFound(Uuid),

    #[error("stream already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
