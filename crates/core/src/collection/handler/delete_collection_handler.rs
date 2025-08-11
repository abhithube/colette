use crate::{
    Handler, RepositoryError,
    collection::{CollectionError, CollectionId, CollectionRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct DeleteCollectionCommand {
    pub id: CollectionId,
    pub user_id: UserId,
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

    async fn handle(&self, cmd: DeleteCollectionCommand) -> Result<Self::Response, Self::Error> {
        let collection = self
            .collection_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| DeleteCollectionError::NotFound(cmd.id))?;
        collection.authorize(cmd.user_id)?;

        self.collection_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(CollectionId),

    #[error(transparent)]
    Core(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
