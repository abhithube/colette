pub use auth_service::*;
use colette_util::password;

use crate::{account, user};

mod auth_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Users(#[from] user::Error),

    #[error(transparent)]
    Accounts(#[from] account::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Hash(#[from] password::Error),
}
