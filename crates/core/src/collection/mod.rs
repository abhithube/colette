use chrono::{DateTime, Utc};
pub use collection_repository::*;
pub use collection_service::*;
use uuid::Uuid;

use crate::{
    bookmark::BookmarkFilter,
    common::{Cursor, CursorError},
};

mod collection_repository;
mod collection_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Collection {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CollectionCursor {
    pub title: String,
}

impl Cursor for Collection {
    type Data = CollectionCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
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
    Cursor(#[from] CursorError),

    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),

    #[error(transparent)]
    SqlitePool(#[from] deadpool_sqlite::PoolError),

    #[error(transparent)]
    SqliteClient(#[from] rusqlite::Error),
}
