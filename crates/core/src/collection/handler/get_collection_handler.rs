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
