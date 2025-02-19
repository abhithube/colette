use chrono::{DateTime, Utc};
pub use collection_repository::*;
pub use collection_service::*;
use uuid::Uuid;

mod collection_repository;
mod collection_service;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub folder_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
