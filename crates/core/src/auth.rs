use crate::{profiles, users};

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
    Profiles(#[from] profiles::Error),

    #[error(transparent)]
    Users(#[from] users::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
