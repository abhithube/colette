use uuid::Uuid;

use super::{Collection, CollectionFindParams, CollectionRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetCollectionQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetCollectionHandler {
    collection_repository: Box<dyn CollectionRepository>,
}

impl GetCollectionHandler {
    pub fn new(collection_repository: impl CollectionRepository) -> Self {
        Self {
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetCollectionQuery> for GetCollectionHandler {
    type Response = Collection;
    type Error = GetCollectionError;

    async fn handle(&self, query: GetCollectionQuery) -> Result<Self::Response, Self::Error> {
        let mut collections = self
            .collection_repository
            .find(CollectionFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if collections.is_empty() {
            return Err(GetCollectionError::NotFound(query.id));
        }

        let collection = collections.swap_remove(0);
        if collection.user_id != query.user_id {
            return Err(GetCollectionError::Forbidden(collection.id));
        }

        Ok(collection)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access collection with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
