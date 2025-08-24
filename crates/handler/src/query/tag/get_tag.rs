use colette_core::{
    auth::UserId,
    common::RepositoryError,
    tag::{TagDto, TagError, TagFindParams, TagId, TagRepository},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct GetTagQuery {
    pub id: TagId,
    pub user_id: UserId,
}

pub struct GetTagHandler<TR: TagRepository> {
    tag_repository: TR,
}

impl<TR: TagRepository> GetTagHandler<TR> {
    pub fn new(tag_repository: TR) -> Self {
        Self { tag_repository }
    }
}

#[async_trait::async_trait]
impl<TR: TagRepository> Handler<GetTagQuery> for GetTagHandler<TR> {
    type Response = TagDto;
    type Error = GetTagError;

    async fn handle(&self, query: GetTagQuery) -> Result<Self::Response, Self::Error> {
        let mut tags = self
            .tag_repository
            .find(TagFindParams {
                user_id: query.user_id,
                id: Some(query.id),
                cursor: None,
                limit: None,
            })
            .await?;
        if tags.is_empty() {
            return Err(GetTagError::Tag(TagError::NotFound(query.id)));
        }

        Ok(tags.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetTagError {
    #[error(transparent)]
    Tag(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
