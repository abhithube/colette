pub use account_repository::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

mod account_repository;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub sub: String,
    pub provider: String,
    pub password_hash: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
