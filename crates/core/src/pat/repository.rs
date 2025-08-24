use crate::{
    auth::UserId,
    common::RepositoryError,
    pat::{LookupHash, PatByLookupHash, PatId, PersonalAccessToken},
};

#[async_trait::async_trait]
pub trait PatRepository: Send + Sync + 'static {
    async fn find_by_id(
        &self,
        id: PatId,
        user_id: UserId,
    ) -> Result<Option<PersonalAccessToken>, RepositoryError>;

    async fn find_by_lookup_hash(
        &self,
        lookup_hash: &LookupHash,
    ) -> Result<Option<PatByLookupHash>, RepositoryError>;

    async fn count(&self, user_id: UserId) -> Result<u8, RepositoryError>;

    async fn save(&self, data: &PersonalAccessToken) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: PatId, user_id: UserId) -> Result<(), RepositoryError>;
}
