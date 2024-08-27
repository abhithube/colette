use email_address::EmailAddress;

use crate::{common::NonEmptyString, profile, user};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Register {
    pub email: EmailAddress,
    pub password: NonEmptyString,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Login {
    pub email: EmailAddress,
    pub password: NonEmptyString,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Profiles(#[from] profile::Error),

    #[error(transparent)]
    Users(#[from] user::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
