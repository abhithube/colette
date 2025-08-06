use url::Url;
use uuid::Uuid;

use super::{Error, User};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error>;

    async fn find_by_email(&self, email: String) -> Result<Option<User>, Error>;

    async fn insert(&self, params: UserInsertParams) -> Result<Uuid, Error>;

    async fn update(&self, params: UserUpdateParams) -> Result<(), Error>;
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
    pub id: Uuid,
    pub display_name: Option<Option<String>>,
    pub image_url: Option<Option<Url>>,
}
