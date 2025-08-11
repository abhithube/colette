use crate::{
    RepositoryError,
    bookmark::BookmarkFilter,
    collection::{Collection, CollectionId},
    user::UserId,
};

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync + 'static {
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, RepositoryError>;

    async fn find_by_id(&self, id: CollectionId) -> Result<Option<Collection>, RepositoryError> {
        let mut collections = self
            .find(CollectionFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if collections.is_empty() {
            return Ok(None);
        }

        Ok(Some(collections.swap_remove(0)))
    }

    async fn insert(&self, params: CollectionInsertParams)
    -> Result<CollectionId, RepositoryError>;

    async fn update(&self, params: CollectionUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: CollectionId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<CollectionId>,
    pub user_id: Option<UserId>,
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
