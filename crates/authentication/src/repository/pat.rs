use colette_common::RepositoryError;

use crate::{LookupHash, PatByLookupHash, PatId, PersonalAccessToken, UserId};

pub trait PatRepository: Sync {
    fn find_by_id(
        &self,
        id: PatId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<PersonalAccessToken>, RepositoryError>> + Send;

    fn find_by_lookup_hash(
        &self,
        lookup_hash: &LookupHash,
    ) -> impl Future<Output = Result<Option<PatByLookupHash>, RepositoryError>> + Send;

    fn count(&self, user_id: UserId) -> impl Future<Output = Result<u8, RepositoryError>> + Send;

    fn save(
        &self,
        data: &PersonalAccessToken,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

    fn delete_by_id(
        &self,
        id: PatId,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}
