use chrono::{DateTime, Utc};
use colette_util::base64;
pub use collection_repository::*;
pub use collection_service::*;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::bookmark::BookmarkFilter;

mod collection_repository;
mod collection_service;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
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

    #[error("not authorized to access collection with ID: {0}")]
    Forbidden(Uuid),

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Database(#[from] DbErr),
}
