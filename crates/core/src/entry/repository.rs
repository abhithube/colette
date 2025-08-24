use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::entry::{Entry, EntryId};

#[async_trait::async_trait]
pub trait EntryRepository: Sync {
    async fn find_by_id(
        &self,
        id: EntryId,
        user_id: UserId,
    ) -> Result<Option<Entry>, RepositoryError>;

    async fn save(&self, data: &Entry) -> Result<(), RepositoryError>;
}
