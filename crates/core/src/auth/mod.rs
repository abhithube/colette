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
    Argon2(#[from] colette_util::Argon2Error),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),
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
