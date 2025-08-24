use colette_common::RepositoryError;
use colette_crud::EntryError;
use uuid::Uuid;

use crate::{EntryDto, EntryQueryRepository, Handler};

#[derive(Debug, Clone)]
pub struct GetEntryQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetEntryHandler<EQR: EntryQueryRepository> {
    entry_query_repository: EQR,
}

impl<EQR: EntryQueryRepository> GetEntryHandler<EQR> {
    pub fn new(entry_query_repository: EQR) -> Self {
        Self {
            entry_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<EQR: EntryQueryRepository> Handler<GetEntryQuery> for GetEntryHandler<EQR> {
    type Response = EntryDto;
    type Error = GetEntryError;

    async fn handle(&self, query: GetEntryQuery) -> Result<Self::Response, Self::Error> {
        let entry = self
            .entry_query_repository
            .query_by_id(query.id, query.user_id)
            .await?
            .ok_or(EntryError::NotFound(query.id))?;

        Ok(entry)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetEntryError {
    #[error(transparent)]
    Entry(#[from] EntryError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
