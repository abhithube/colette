use super::{CreateData, Error, User};
use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    async fn create(&self, data: CreateData) -> Result<User, Error>;
}
