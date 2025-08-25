use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{
    BookmarkFilter, Collection, CollectionError, CollectionId, CollectionRepository,
    CollectionTitle,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct UpdateCollectionCommand {
    pub id: CollectionId,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
    pub user_id: UserId,
}

pub struct UpdateCollectionHandler<CR: CollectionRepository> {
    collection_repository: CR,
}

impl<CR: CollectionRepository> UpdateCollectionHandler<CR> {
    pub fn new(collection_repository: CR) -> Self {
        Self {
            collection_repository,
        }
    }
}

impl<CR: CollectionRepository> Handler<UpdateCollectionCommand> for UpdateCollectionHandler<CR> {
    type Response = Collection;
    type Error = UpdateCollectionError;

    async fn handle(&self, cmd: UpdateCollectionCommand) -> Result<Self::Response, Self::Error> {
        let mut collection = self
            .collection_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or(CollectionError::NotFound(cmd.id.as_inner()))?;

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
    #[error(transparent)]
    Collection(#[from] CollectionError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
