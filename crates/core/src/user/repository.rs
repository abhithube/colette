use url::Url;

use crate::{
    common::RepositoryError,
    user::{User, UserId},
};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    async fn find_by_email(&self, email: String) -> Result<Option<User>, RepositoryError>;

    async fn insert(&self, params: UserInsertParams) -> Result<UserId, RepositoryError>;

    async fn update(&self, params: UserUpdateParams) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct UserInsertParams {
    pub email: String,
    pub display_name: Option<String>,
    pub image_url: Option<Url>,

    pub sub: String,
    pub provider: String,
    pub password_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserUpdateParams {
    pub id: UserId,
    pub display_name: Option<Option<String>>,
    pub image_url: Option<Option<Url>>,
}
