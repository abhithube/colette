use colette_authentication::UserId;
use colette_common::RepositoryError;

use crate::{
    bookmark::BookmarkFilter,
    collection::{Collection, CollectionId},
};

#[async_trait::async_trait]
pub trait CollectionRepository: Sync {
    async fn find_by_id(
        &self,
        id: CollectionId,
        user_id: UserId,
    ) -> Result<Option<Collection>, RepositoryError>;

    async fn save(&self, data: &Collection) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: CollectionId, user_id: UserId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct CollectionInsertParams {
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct CollectionUpdateParams {
    pub id: CollectionId,
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}
