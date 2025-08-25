use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::{Entry, EntryId};

pub trait EntryRepository: Sync {
    fn find_by_id(
        &self,
        id: EntryId,
        user_id: UserId,
    ) -> impl std::future::Future<Output = Result<Option<Entry>, RepositoryError>> + Send;

    fn save(
        &self,
        data: &Entry,
    ) -> impl std::future::Future<Output = Result<(), RepositoryError>> + Send;
}
