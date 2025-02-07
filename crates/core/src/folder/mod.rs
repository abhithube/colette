pub use folder_repository::*;
pub use folder_service::*;
use uuid::Uuid;

mod folder_repository;
mod folder_service;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("folder not found with ID: {0}")]
    NotFound(Uuid),

    #[error("folder already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
