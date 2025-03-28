pub use backup_repository::*;
pub use backup_service::*;

use crate::job;

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
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
}
