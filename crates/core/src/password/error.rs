use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to hash password")]
    Hash,

    #[error("failed to verify password")]
    Verify,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
