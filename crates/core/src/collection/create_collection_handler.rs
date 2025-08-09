use uuid::Uuid;

use super::{CollectionInsertParams, CollectionRepository};
use crate::{Handler, RepositoryError, bookmark::BookmarkFilter};

#[derive(Debug, Clone)]
pub struct CreateCollectionCommand {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
}

pub struct CreateCollectionHandler {
    collection_repository: Box<dyn CollectionRepository>,
}

impl CreateCollectionHandler {
    pub fn new(collection_repository: impl CollectionRepository) -> Self {
        Self {
            collection_repository: Box::new(collection_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<CreateCollectionCommand> for CreateCollectionHandler {
    type Response = CollectionCreated;
    type Error = CreateCollectionError;

    async fn handle(&self, data: CreateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .collection_repository
            .insert(CollectionInsertParams {
                title: data.title,
                filter: data.filter,
                user_id: data.user_id,
            })
            .await?;

        Ok(CollectionCreated { id })
    }
}

#[derive(Debug, Clone)]
pub struct CollectionCreated {
    pub id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateCollectionError {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access collection with ID: {0}")]
    Forbidden(Uuid),

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
