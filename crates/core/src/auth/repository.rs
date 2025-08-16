use chrono::{DateTime, Utc};
use email_address::EmailAddress;

use crate::{
    User,
    auth::{LookupHash, PatByLookupHash, PatId, PersonalAccessToken, UserId},
    common::RepositoryError,
};

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

#[async_trait::async_trait]
pub trait PatRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: PatFindParams,
    ) -> Result<Vec<PersonalAccessToken>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: PatId,
        user_id: UserId,
    ) -> Result<Option<PersonalAccessToken>, RepositoryError> {
        let mut pats = self
            .find(PatFindParams {
                id: Some(id),
                user_id,
                cursor: None,
                limit: None,
            })
            .await?;
        if pats.is_empty() {
            return Ok(None);
        }

        Ok(Some(pats.swap_remove(0)))
    }

    async fn find_by_lookup_hash(
        &self,
        lookup_hash: &LookupHash,
    ) -> Result<Option<PatByLookupHash>, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PatFindParams {
    pub user_id: UserId,
    pub id: Option<PatId>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}
