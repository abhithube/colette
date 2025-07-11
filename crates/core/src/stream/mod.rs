use chrono::{DateTime, Utc};
pub use stream_repository::*;
pub use stream_service::*;
use uuid::Uuid;

use crate::{
    common::{Cursor, CursorError},
    subscription_entry::SubscriptionEntryFilter,
};

mod stream_repository;
mod stream_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct Stream {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub title: String,
    pub filter: SubscriptionEntryFilter,
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StreamCursor {
    pub title: String,
}

impl Cursor for Stream {
    type Data = StreamCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stream not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access stream with ID: {0}")]
    Forbidden(Uuid),

    #[error("stream already exists with title: {0}")]
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
