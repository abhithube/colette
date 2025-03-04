use chrono::{DateTime, Utc};
use colette_util::base64;
pub use feed_entry_repository::*;
pub use feed_entry_service::*;
use sea_orm::DbErr;
use url::Url;
use uuid::Uuid;

use crate::{
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    stream,
};

mod feed_entry_repository;
mod feed_entry_service;

#[derive(Debug, Clone)]
pub struct FeedEntry {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub has_read: bool,
    pub feed_id: Uuid,
    pub user_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub id: Uuid,
    pub published_at: DateTime<Utc>,
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access feed entry with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Stream(#[from] stream::Error),

    #[error(transparent)]
    Database(#[from] DbErr),
}
