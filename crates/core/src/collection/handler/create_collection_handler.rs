use crate::{
    Collection, Handler,
    auth::UserId,
    bookmark::BookmarkFilter,
    collection::{CollectionError, CollectionRepository, CollectionTitle},
    common::RepositoryError,
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
    type Response = Collection;
    type Error = CreateCollectionError;

    async fn handle(&self, cmd: CreateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let title = CollectionTitle::new(cmd.title.clone())?;

        let collection = Collection::new(title, cmd.filter, cmd.user_id);

        self.collection_repository
            .save(&collection)
            .await
            .map_err(|e| match e {
                RepositoryError::Duplicate => {
                    CreateCollectionError::Collection(CollectionError::Conflict(cmd.title))
                }
                _ => CreateCollectionError::Repository(e),
            })?;

        Ok(collection)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateCollectionError {
    #[error(transparent)]
    Collection(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
