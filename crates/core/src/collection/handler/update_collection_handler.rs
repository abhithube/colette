use crate::{
    Collection, Handler,
    auth::UserId,
    bookmark::BookmarkFilter,
    collection::{CollectionError, CollectionId, CollectionRepository, CollectionTitle},
    common::RepositoryError,
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
    type Response = Collection;
    type Error = UpdateCollectionError;

    async fn handle(&self, cmd: UpdateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let mut collection = self
            .collection_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or_else(|| UpdateCollectionError::NotFound(cmd.id))?;

        if let Some(title) = cmd.title.map(CollectionTitle::new).transpose()? {
            collection.set_title(title);
        }
        if let Some(filter) = cmd.filter {
            collection.set_filter(filter);
        }

        self.collection_repository.save(&collection).await?;

        Ok(collection)
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
