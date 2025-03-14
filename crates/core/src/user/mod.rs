use chrono::{DateTime, Utc};
use sea_orm::DbErr;
pub use user_repository::*;
use uuid::Uuid;

mod user_repository;

#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Database(#[from] DbErr),
}
