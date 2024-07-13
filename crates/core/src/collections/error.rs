use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("collection not found with id: {0}")]
    NotFound(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
