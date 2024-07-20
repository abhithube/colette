use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait UsersRepository: Send + Sync {
    async fn find_one(&self, params: UserFindOneParams) -> Result<User, Error>;

    async fn create(&self, data: UserCreateData) -> Result<User, Error>;
}

#[derive(Clone, Debug)]
pub struct UserFindOneParams {
    pub email: String,
}

#[derive(Clone, Debug)]
pub struct UserCreateData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found with email: {0}")]
    NotFound(String),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
