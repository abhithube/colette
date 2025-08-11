use crate::{
    Handler,
    common::RepositoryError,
    tag::{Tag, TagError, TagFindParams, TagId, TagRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct GetTagQuery {
    pub id: TagId,
    pub user_id: UserId,
}

pub struct GetTagHandler {
    tag_repository: Box<dyn TagRepository>,
}

impl GetTagHandler {
    pub fn new(tag_repository: impl TagRepository) -> Self {
        Self {
            tag_repository: Box::new(tag_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetTagQuery> for GetTagHandler {
    type Response = Tag;
    type Error = GetTagError;

    async fn handle(&self, query: GetTagQuery) -> Result<Self::Response, Self::Error> {
        let mut tags = self
            .tag_repository
            .find(TagFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if tags.is_empty() {
            return Err(GetTagError::NotFound(query.id));
        }

        let tag = tags.swap_remove(0);
        tag.authorize(query.user_id)?;

        Ok(tag)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetTagError {
    #[error("tag not found with ID: {0}")]
    NotFound(TagId),

    #[error(transparent)]
    Core(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
