pub use backup_repository::*;
pub use backup_service::*;

use crate::{job, queue};

mod backup_repository;
mod backup_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Opml(#[from] colette_opml::Error),

    #[error(transparent)]
    Netscape(#[from] colette_netscape::Error),

    #[error(transparent)]
    Job(#[from] job::Error),

    #[error(transparent)]
    Queue(#[from] queue::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
