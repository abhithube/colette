use crate::{profile, user};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Register {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
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
