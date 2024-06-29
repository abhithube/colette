use super::{types::FindOneParams, CreateData, Error, User};
use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    async fn find_one(&self, params: FindOneParams) -> Result<User, Error>;

    async fn create(&self, data: CreateData) -> Result<User, Error>;
}
