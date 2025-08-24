use colette_core::{common::RepositoryError, tag::TagError};
use uuid::Uuid;

use crate::{Handler, TagDto, TagQueryRepository};

#[derive(Debug, Clone)]
pub struct GetTagQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetTagHandler<TQR: TagQueryRepository> {
    tag_query_repository: TQR,
}

impl<TQR: TagQueryRepository> GetTagHandler<TQR> {
    pub fn new(tag_query_repository: TQR) -> Self {
        Self {
            tag_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<TQR: TagQueryRepository> Handler<GetTagQuery> for GetTagHandler<TQR> {
    type Response = TagDto;
    type Error = GetTagError;

    async fn handle(&self, query: GetTagQuery) -> Result<Self::Response, Self::Error> {
        let tag = self
            .tag_query_repository
            .query_by_id(query.id, query.user_id)
            .await?
            .ok_or(TagError::NotFound(query.id))?;

        Ok(tag)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetTagError {
    #[error(transparent)]
    Tag(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
