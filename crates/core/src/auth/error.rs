use thiserror::Error;

use crate::{profiles, users};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Profiles(#[from] profiles::Error),

    #[error(transparent)]
    Users(#[from] users::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
