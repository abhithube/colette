use crate::{
    Handler,
    bookmark::BookmarkFilter,
    collection::{CollectionError, CollectionId, CollectionRepository, CollectionUpdateParams},
    common::RepositoryError,
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct UpdateCollectionCommand {
    pub id: CollectionId,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
    pub user_id: UserId,
}

pub struct UpdateCollectionHandler {
    collection_repository: Box<dyn CollectionRepository>,
}

impl UpdateCollectionHandler {
    pub fn new(collection_repository: impl CollectionRepository) -> Self {
        Self {
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<UpdateCollectionCommand> for UpdateCollectionHandler {
    type Response = ();
    type Error = UpdateCollectionError;

    async fn handle(&self, cmd: UpdateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let collection = self
            .collection_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| UpdateCollectionError::NotFound(cmd.id))?;
        collection.authorize(cmd.user_id)?;

        self.collection_repository
            .update(CollectionUpdateParams {
                id: cmd.id,
                title: cmd.title,
                filter: cmd.filter,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(CollectionId),

    #[error(transparent)]
    Core(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
