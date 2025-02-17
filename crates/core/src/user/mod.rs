use chrono::{DateTime, Utc};
pub use user_repository::*;
use uuid::Uuid;

mod user_repository;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum NotFoundError {
    #[error("user not found with id: {0}")]
    Id(Uuid),

    #[error("user not found with email: {0}")]
    Email(String),
}
