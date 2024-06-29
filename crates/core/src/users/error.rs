use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
