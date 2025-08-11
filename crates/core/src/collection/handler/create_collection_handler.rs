use crate::{
    Handler, RepositoryError,
    bookmark::BookmarkFilter,
    collection::{CollectionId, CollectionInsertParams, CollectionRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct CreateCollectionCommand {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: UserId,
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

    async fn handle(&self, cmd: CreateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .collection_repository
            .insert(CollectionInsertParams {
                title: cmd.title.clone(),
                filter: cmd.filter,
                user_id: cmd.user_id,
            })
            .await
            .map_err(|e| match e {
                RepositoryError::Duplicate => CreateCollectionError::Conflict(cmd.title),
                _ => CreateCollectionError::Repository(e),
            })?;

        Ok(CollectionCreated { id })
    }
}

#[derive(Debug, Clone)]
pub struct CollectionCreated {
    pub id: CollectionId,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateCollectionError {
    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
