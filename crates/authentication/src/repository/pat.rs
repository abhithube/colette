use colette_common::RepositoryError;

use crate::{LookupHash, PatByLookupHash, PatId, PersonalAccessToken, UserId};

#[async_trait::async_trait]
pub trait PatRepository: Sync {
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
