use uuid::Uuid;

use super::{Error, User};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_user(&self, params: UserFindParams) -> Result<User, Error>;
}

#[derive(Debug, Clone)]
pub struct UserFindParams {
    pub id: Uuid,
}
