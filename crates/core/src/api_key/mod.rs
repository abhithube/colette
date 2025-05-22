pub use api_key_repository::*;
pub use api_key_service::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

mod api_key_repository;
mod api_key_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct ApiKey {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub lookup_hash: String,
    pub verification_hash: String,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
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

    #[error("invalid API key")]
    Auth,

    #[error(transparent)]
    Hash(#[from] colette_util::Argon2Error),

    #[error(transparent)]
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
}
