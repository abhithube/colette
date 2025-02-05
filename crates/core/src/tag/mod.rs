pub use tag_repository::*;
pub use tag_service::*;
use uuid::Uuid;

mod tag_repository;
mod tag_service;

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    pub bookmark_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub enum TagType {
    #[default]
    All,
    Bookmarks,
    Feeds,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
