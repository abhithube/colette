pub use api_key_repository::*;
pub use api_key_service::*;
use chrono::{DateTime, Utc};
use colette_util::password;
use uuid::Uuid;

use crate::auth;

mod api_key_repository;
mod api_key_service;

#[derive(Debug, Clone, Default)]
pub struct ApiKey {
    pub id: Uuid,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API key not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access API key with ID: {0}")]
    Forbidden(Uuid),

    #[error("API key already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Hash(#[from] password::Error),

    #[error(transparent)]
    Auth(#[from] auth::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
