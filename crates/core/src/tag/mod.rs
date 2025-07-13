use chrono::{DateTime, Utc};
pub use tag_repository::*;
pub use tag_service::*;
use uuid::Uuid;

use crate::pagination::Cursor;

mod tag_repository;
mod tag_service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bon::Builder)]
pub struct Tag {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark_count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagType {
    Bookmarks,
    Feeds,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TagCursor {
    pub title: String,
}

impl Cursor for Tag {
    type Data = TagCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access tag with ID: {0}")]
    Forbidden(Uuid),

    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),

    #[error(transparent)]
    SqlitePool(#[from] deadpool_sqlite::PoolError),

    #[error(transparent)]
    SqliteClient(#[from] rusqlite::Error),
}
