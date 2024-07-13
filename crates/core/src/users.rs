use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found with email: {0}")]
    NotFound(String),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait UsersRepository {
    async fn find_one(&self, params: UserFindOneParams) -> Result<User, Error>;

    async fn create(&self, data: UserCreateData) -> Result<User, Error>;
}

pub struct UserFindOneParams {
    pub email: String,
}

pub struct UserCreateData {
    pub email: String,
    pub password: String,
}
