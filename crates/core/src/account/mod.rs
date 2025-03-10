pub use account_repository::*;
use uuid::Uuid;

mod account_repository;

#[derive(Debug, Clone, Default)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub provider_id: String,
    pub account_id: String,
    pub password_hash: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("account not found with id: {0}")]
    NotFound(String),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
