use uuid::Uuid;

use super::{CollectionRepository, CollectionUpdateParams};
use crate::{Handler, RepositoryError, bookmark::BookmarkFilter};

#[derive(Debug, Clone, Default)]
pub struct UpdateCollectionCommand {
    pub id: Uuid,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
    pub user_id: Uuid,
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

    async fn handle(&self, data: UpdateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let Some(collection) = self.collection_repository.find_by_id(data.id).await? else {
            return Err(UpdateCollectionError::NotFound(data.id));
        };
        if collection.user_id != data.user_id {
            return Err(UpdateCollectionError::Forbidden(data.id));
        }

        self.collection_repository
            .update(CollectionUpdateParams {
                id: data.id,
                title: data.title,
                filter: data.filter,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access collection with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
