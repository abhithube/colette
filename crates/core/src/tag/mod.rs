use chrono::{DateTime, Utc};
pub use tag_repository::*;
pub use tag_service::*;
use uuid::Uuid;

mod tag_repository;
mod tag_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Tag {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    pub bookmark_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TagType {
    #[default]
    All,
    Bookmarks,
    Feeds,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
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
    Database(#[from] sqlx::Error),
}
