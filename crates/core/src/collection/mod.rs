use chrono::{DateTime, Utc};
use colette_util::base64;
pub use collection_repository::*;
pub use collection_service::*;
use uuid::Uuid;

use crate::bookmark::BookmarkFilter;

mod collection_repository;
mod collection_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Collection {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: String,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
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
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
}
