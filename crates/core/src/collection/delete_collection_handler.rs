use uuid::Uuid;

use super::CollectionRepository;
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct DeleteCollectionCommand {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct DeleteCollectionHandler {
    collection_repository: Box<dyn CollectionRepository>,
}

impl DeleteCollectionHandler {
    pub fn new(collection_repository: impl CollectionRepository) -> Self {
        Self {
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DeleteCollectionCommand> for DeleteCollectionHandler {
    type Response = ();
    type Error = DeleteCollectionError;

    async fn handle(&self, data: DeleteCollectionCommand) -> Result<Self::Response, Self::Error> {
        let Some(collection) = self.collection_repository.find_by_id(data.id).await? else {
            return Err(DeleteCollectionError::NotFound(data.id));
        };
        if collection.user_id != data.user_id {
            return Err(DeleteCollectionError::Forbidden(data.id));
        }

        self.collection_repository.delete_by_id(data.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access collection with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
