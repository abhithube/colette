use chrono::{DateTime, Utc};
pub use user_repository::*;
use uuid::Uuid;

mod user_repository;

#[derive(Debug, Clone, bon::Builder)]
pub struct User {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub email: String,
    pub verified_at: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub password_hash: Option<String>,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found with id: {0}")]
    NotFound(Uuid),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
