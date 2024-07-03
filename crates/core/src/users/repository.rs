use super::{Error, User};
use async_trait::async_trait;

#[async_trait]
pub trait UsersRepository {
    async fn find_one(&self, params: UserFindOneParams<'_>) -> Result<User, Error>;

    async fn create(&self, data: UserCreateData<'_>) -> Result<User, Error>;
}

pub struct UserFindOneParams<'a> {
    pub email: &'a str,
}

pub struct UserCreateData<'a> {
    pub email: &'a str,
    pub password: &'a str,
}
