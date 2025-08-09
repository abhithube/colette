use uuid::Uuid;

use super::Tag;
use crate::RepositoryError;

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find(&self, params: TagFindParams) -> Result<Vec<Tag>, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<TagById>, RepositoryError>;

    async fn insert(&self, params: TagInsertParams) -> Result<Uuid, RepositoryError>;

    async fn update(&self, params: TagUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct TagFindParams {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TagById {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct TagInsertParams {
    pub title: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct TagUpdateParams {
    pub id: Uuid,
    pub title: Option<String>,
}
