use async_trait::async_trait;

use super::{Error, User};

#[async_trait]
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
