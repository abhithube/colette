use apalis_redis::RedisError;
pub use backup_repository::*;
pub use backup_service::*;
use sea_orm::DbErr;

mod backup_repository;
mod backup_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Opml(#[from] colette_opml::Error),

    #[error(transparent)]
    Netscape(#[from] colette_netscape::Error),

    #[error(transparent)]
    Database(#[from] DbErr),

    #[error(transparent)]
    Redis(#[from] RedisError),
}
