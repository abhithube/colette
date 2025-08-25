use colette_common::RepositoryError;
use colette_crud::CollectionError;
use uuid::Uuid;

use crate::{CollectionDto, CollectionQueryRepository, Handler};

#[derive(Debug, Clone)]
pub struct GetCollectionQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetCollectionHandler<CQR: CollectionQueryRepository> {
    collection_query_repository: CQR,
}

impl<CQR: CollectionQueryRepository> GetCollectionHandler<CQR> {
    pub fn new(collection_query_repository: CQR) -> Self {
        Self {
            collection_query_repository,
        }
    }
}

impl<CQR: CollectionQueryRepository> Handler<GetCollectionQuery> for GetCollectionHandler<CQR> {
    type Response = CollectionDto;
    type Error = GetCollectionError;

    async fn handle(&self, query: GetCollectionQuery) -> Result<Self::Response, Self::Error> {
        let collection = self
            .collection_query_repository
            .query_by_id(query.id, query.user_id)
            .await?
            .ok_or(CollectionError::NotFound(query.id))?;

        Ok(collection)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetCollectionError {
    #[error(transparent)]
    Collection(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
