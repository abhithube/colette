use crate::{
    auth::UserId,
    common::RepositoryError,
    entry::{Entry, EntryId},
};

#[async_trait::async_trait]
pub trait EntryRepository: Send + Sync + 'static {
    async fn find_by_id(
        &self,
        id: EntryId,
        user_id: UserId,
    ) -> Result<Option<Entry>, RepositoryError>;

    async fn save(&self, data: &Entry) -> Result<(), RepositoryError>;
}
