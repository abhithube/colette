use crate::{
    Handler,
    auth::UserId,
    collection::{CollectionError, CollectionId, CollectionRepository},
    common::RepositoryError,
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
        self.collection_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    DeleteCollectionError::Collection(CollectionError::NotFound(cmd.id))
                }
                _ => DeleteCollectionError::Repository(e),
            })?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteCollectionError {
    #[error(transparent)]
    Collection(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
