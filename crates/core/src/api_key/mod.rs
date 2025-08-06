pub use api_key_repository::*;
pub use api_key_service::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::pagination::Cursor;

mod api_key_repository;
mod api_key_service;

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: Uuid,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyCursor {
    pub created_at: DateTime<Utc>,
}

impl Cursor for ApiKey {
    type Data = ApiKeyCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API key not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access API key with ID: {0}")]
    Forbidden(Uuid),

    #[error("invalid API key")]
    Auth,

    #[error(transparent)]
    Crypto(#[from] colette_util::CryptoError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
