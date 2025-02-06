use apalis_redis::RedisError;
pub use backup_repository::*;
pub use backup_service::*;

use crate::{bookmark, feed, folder};

mod backup_repository;
mod backup_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Opml(#[from] colette_opml::Error),

    #[error(transparent)]
    Netscape(#[from] colette_netscape::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Redis(#[from] RedisError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Folder(#[from] folder::Error),

    #[error(transparent)]
    Feed(#[from] feed::Error),

    #[error(transparent)]
    Bookmark(#[from] bookmark::Error),
}
