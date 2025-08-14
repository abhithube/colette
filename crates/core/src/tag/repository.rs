use crate::{
    common::RepositoryError,
    tag::{Tag, TagId},
    auth::UserId,
};

#[async_trait::async_trait]
pub trait TagRepository: Send + Sync + 'static {
    async fn find(&self, params: TagFindParams) -> Result<Vec<Tag>, RepositoryError>;

    async fn find_by_id(&self, id: TagId) -> Result<Option<Tag>, RepositoryError> {
        let mut tags = self
            .find(TagFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Ok(None);
        }

        Ok(Some(tags.swap_remove(0)))
    }

    async fn insert(&self, params: TagInsertParams) -> Result<TagId, RepositoryError>;

    async fn update(&self, params: TagUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: TagId) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct TagFindParams {
    pub id: Option<TagId>,
    pub user_id: Option<UserId>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TagInsertParams {
    pub title: String,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct TagUpdateParams {
    pub id: TagId,
    pub title: Option<String>,
}
