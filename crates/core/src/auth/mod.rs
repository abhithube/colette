pub use auth_service::*;
use colette_util::password;

use crate::user;

mod auth_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Users(#[from] user::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Hash(#[from] password::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
