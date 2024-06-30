use super::{Error, User, UserCreateData, UserFindOneParams};
use async_trait::async_trait;

#[async_trait]
pub trait UsersRepository {
    async fn find_one(&self, params: UserFindOneParams) -> Result<User, Error>;

    async fn create(&self, data: UserCreateData) -> Result<User, Error>;
}
