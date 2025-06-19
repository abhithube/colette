pub use auth_service::*;

use crate::{account, user};

mod auth_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    User(user::Error),

    #[error(transparent)]
    Account(account::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error("Missing JWT key ID")]
    MissingKid,

    #[error("Missing JWK")]
    MissingJwk,

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Crypto(#[from] colette_util::CryptoError),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

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

impl From<user::Error> for Error {
    fn from(value: user::Error) -> Self {
        Error::User(value)
    }
}

impl From<account::Error> for Error {
    fn from(value: account::Error) -> Self {
        Error::Account(value)
    }
}
