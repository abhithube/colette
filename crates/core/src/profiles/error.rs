use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("profile not found with id: {0}")]
    NotFound(String),

    #[error("default profile cannot be deleted")]
    DeletingDefault,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
