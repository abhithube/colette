use crate::{
    auth::UserId,
    common::RepositoryError,
    tag::{Tag, TagDto, TagId},
};

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find(&self, params: TagFindParams) -> Result<Vec<TagDto>, RepositoryError>;

    async fn find_by_id(&self, id: TagId, user_id: UserId) -> Result<Option<Tag>, RepositoryError>;

    async fn save(&self, data: &Tag) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: TagId, user_id: UserId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct TagFindParams {
    pub user_id: UserId,
    pub id: Option<TagId>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}
