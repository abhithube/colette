use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("user not found with email: {0}")]
    NotFound(String),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
