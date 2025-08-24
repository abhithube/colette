use colette_common::RepositoryError;
use email_address::EmailAddress;

use crate::{User, UserId};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    async fn find_by_email(&self, email: EmailAddress) -> Result<Option<User>, RepositoryError>;

    async fn find_by_provider_and_sub(
        &self,
        provider: String,
        sub: String,
    ) -> Result<Option<User>, RepositoryError>;

    async fn save(&self, data: &User) -> Result<(), RepositoryError>;
}
