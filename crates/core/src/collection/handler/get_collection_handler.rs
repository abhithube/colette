use crate::{
    Handler,
    auth::UserId,
    collection::{
        CollectionDto, CollectionError, CollectionFindParams, CollectionId, CollectionRepository,
    },
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct GetCollectionQuery {
    pub id: CollectionId,
    pub user_id: UserId,
}

pub struct GetCollectionHandler<CR: CollectionRepository> {
    collection_repository: CR,
}

impl<CR: CollectionRepository> GetCollectionHandler<CR> {
    pub fn new(collection_repository: CR) -> Self {
        Self {
            collection_repository,
        }
    }
}

#[async_trait::async_trait]
impl<CR: CollectionRepository> Handler<GetCollectionQuery> for GetCollectionHandler<CR> {
    type Response = CollectionDto;
    type Error = GetCollectionError;

    async fn handle(&self, query: GetCollectionQuery) -> Result<Self::Response, Self::Error> {
        let mut collections = self
            .collection_repository
            .find(CollectionFindParams {
                user_id: query.user_id,
                id: Some(query.id),
                cursor: None,
                limit: None,
            })
            .await?;
        if collections.is_empty() {
            return Err(GetCollectionError::NotFound(query.id));
        }

        Ok(collections.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(CollectionId),

    #[error(transparent)]
    Core(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
