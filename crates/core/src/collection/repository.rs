use crate::{
    auth::UserId,
    bookmark::BookmarkFilter,
    collection::{Collection, CollectionDto, CollectionId},
    common::RepositoryError,
};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn find(
        &self,
        params: CollectionFindParams,
    ) -> Result<Vec<CollectionDto>, RepositoryError>;

    async fn find_by_id(
        &self,
        id: CollectionId,
        user_id: UserId,
    ) -> Result<Option<Collection>, RepositoryError>;

    async fn save(&self, data: &Collection) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: CollectionId, user_id: UserId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct CollectionFindParams {
    pub user_id: UserId,
    pub id: Option<CollectionId>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
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
