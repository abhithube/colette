use chrono::{DateTime, Utc};
pub use feed_entry_repository::*;
pub use feed_entry_service::*;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{Cursor, CursorError},
    stream,
};

mod feed_entry_repository;
mod feed_entry_service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bon::Builder)]
pub struct FeedEntry {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub feed_id: Uuid,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedEntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor for FeedEntry {
    type Data = FeedEntryCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            published_at: self.published_at,
            id: self.id,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access feed entry with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Stream(#[from] stream::Error),

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
