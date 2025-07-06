pub use backup_repository::*;
pub use backup_service::*;

use crate::{Bookmark, Subscription, Tag, bookmark, subscription, tag};

mod backup_repository;
mod backup_service;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Backup {
    pub bookmarks: Vec<Bookmark>,
    pub subscriptions: Vec<Subscription>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Subscription(#[from] subscription::Error),

    #[error(transparent)]
    Bookmark(#[from] bookmark::Error),

    #[error(transparent)]
    Tag(#[from] tag::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),

    #[error(transparent)]
    SqlitePool(#[from] deadpool_sqlite::PoolError),

    #[error(transparent)]
    SqliteClient(#[from] rusqlite::Error),
}
