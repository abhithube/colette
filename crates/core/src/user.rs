use uuid::Uuid;

use crate::common::Creatable;

#[derive(Clone, Debug, serde::Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
}

#[async_trait::async_trait]
pub trait UserRepository:
    Creatable<Data = UserCreateData, Output = Result<User, Error>> + Send + Sync
{
    async fn find_one(&self, params: UserIdParams) -> Result<User, Error>;
}

#[derive(Clone, Debug)]
pub enum UserIdParams {
    Id(Uuid),
    Email(String),
}

#[derive(Clone, Debug)]
pub struct UserCreateData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NotFound(#[from] NotFoundError),

    #[error("user already exists with email: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum NotFoundError {
    #[error("user not found with id: {0}")]
    Id(Uuid),

    #[error("user not found with email: {0}")]
    Email(String),
}
