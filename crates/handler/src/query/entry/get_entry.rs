use colette_core::{
    auth::UserId,
    common::RepositoryError,
    entry::{EntryDto, EntryError, EntryFindParams, EntryId, EntryRepository},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct GetEntryQuery {
    pub id: EntryId,
    pub user_id: UserId,
}

pub struct GetEntryHandler<ER: EntryRepository> {
    entry_repository: ER,
}

impl<ER: EntryRepository> GetEntryHandler<ER> {
    pub fn new(entry_repository: ER) -> Self {
        Self { entry_repository }
    }
}

#[async_trait::async_trait]
impl<ER: EntryRepository> Handler<GetEntryQuery> for GetEntryHandler<ER> {
    type Response = EntryDto;
    type Error = GetEntryError;

    async fn handle(&self, query: GetEntryQuery) -> Result<Self::Response, Self::Error> {
        let mut entries = self
            .entry_repository
            .find(EntryFindParams {
                user_id: query.user_id,
                id: Some(query.id),
                subscription_id: None,
                has_read: None,
                tags: None,
                filter: None,
                cursor: None,
                limit: None,
            })
            .await?;
        if entries.is_empty() {
            return Err(GetEntryError::Entry(EntryError::NotFound(query.id)));
        }

        Ok(entries.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetEntryError {
    #[error(transparent)]
    Entry(#[from] EntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
