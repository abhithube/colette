use uuid::Uuid;

use super::{Error, User};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_one(&self, key: UserFindOne) -> Result<Option<User>, Error>;

    async fn save(&self, data: &User) -> Result<(), Error>;
}

pub enum UserFindOne {
    Id(Uuid),
    Email(String),
}
