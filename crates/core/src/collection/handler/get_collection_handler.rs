use crate::{
    Handler,
    collection::{
        Collection, CollectionError, CollectionFindParams, CollectionId, CollectionRepository,
    },
    common::RepositoryError,
    user::UserId,
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
        collection.authorize(query.user_id)?;

        Ok(collection)
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
