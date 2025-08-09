use uuid::Uuid;

use super::{Tag, TagFindParams, TagRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetTagQuery {
    pub id: Uuid,
    pub user_id: Uuid,
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
        if tag.user_id != query.user_id {
            return Err(GetTagError::Forbidden(query.id));
        }

        Ok(tag)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetTagError {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access tag with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
