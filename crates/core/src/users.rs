use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
}

#[async_trait::async_trait]
pub trait UsersRepository: Send + Sync {
    async fn find_one(&self, params: UsersFindOneParams) -> Result<User, Error>;

    async fn create(&self, data: UsersCreateData) -> Result<User, Error>;
}

#[derive(Clone, Debug)]
pub struct UsersFindOneParams {
    pub email: String,
}

#[derive(Clone, Debug)]
pub struct UsersCreateData {
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
