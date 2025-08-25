use colette_common::RepositoryError;
use email_address::EmailAddress;

use crate::{User, UserId};

pub trait UserRepository: Sync {
    fn find_by_id(
        &self,
        id: UserId,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn find_by_email(
        &self,
        email: EmailAddress,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn find_by_provider_and_sub(
        &self,
        provider: String,
        sub: String,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn save(&self, data: &User) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}
