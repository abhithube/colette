pub use account_repository::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::User;

mod account_repository;

#[derive(Debug, Clone, serde::Deserialize, bon::Builder)]
pub struct Account {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub sub: String,
    pub provider: String,
    pub password_hash: Option<String>,
    pub user_id: Uuid,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
    pub user: Option<User>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PostgresPool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    PostgresClient(#[from] tokio_postgres::Error),
}
