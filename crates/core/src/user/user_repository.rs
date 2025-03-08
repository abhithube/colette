use uuid::Uuid;

use super::{Error, User};
use crate::common::Transaction;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_user(&self, params: UserFindParams) -> Result<User, Error>;

    async fn create_user(
        &self,
        tx: &dyn Transaction,
        params: UserCreateParams,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct UserFindParams {
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct UserCreateParams {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}
