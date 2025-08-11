#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Resource not found")]
    NotFound,

    #[error("Duplicate resource")]
    Duplicate,

    #[error(transparent)]
    Unknown(#[from] sqlx::Error),
}
