use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{CollectionError, CollectionId, CollectionRepository};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DeleteCollectionCommand {
    pub id: CollectionId,
    pub user_id: UserId,
}

pub struct DeleteCollectionHandler<CR: CollectionRepository> {
    collection_repository: CR,
}

impl<CR: CollectionRepository> DeleteCollectionHandler<CR> {
    pub fn new(collection_repository: CR) -> Self {
        Self {
            collection_repository,
        }
    }
}

#[async_trait::async_trait]
impl<CR: CollectionRepository> Handler<DeleteCollectionCommand> for DeleteCollectionHandler<CR> {
    type Response = ();
    type Error = DeleteCollectionError;

    async fn handle(&self, cmd: DeleteCollectionCommand) -> Result<Self::Response, Self::Error> {
        self.collection_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    DeleteCollectionError::Collection(CollectionError::NotFound(cmd.id.as_inner()))
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
