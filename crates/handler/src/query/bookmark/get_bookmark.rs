use colette_core::{bookmark::BookmarkError, common::RepositoryError};
use uuid::Uuid;

use crate::{BookmarkDto, BookmarkQueryRepository, Handler};

#[derive(Debug, Clone)]
pub struct GetBookmarkQuery {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct GetBookmarkHandler<BQR: BookmarkQueryRepository> {
    bookmark_query_repository: BQR,
}

impl<BQR: BookmarkQueryRepository> GetBookmarkHandler<BQR> {
    pub fn new(bookmark_query_repository: BQR) -> Self {
        Self {
            bookmark_query_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BQR: BookmarkQueryRepository> Handler<GetBookmarkQuery> for GetBookmarkHandler<BQR> {
    type Response = BookmarkDto;
    type Error = GetBookmarkError;

    async fn handle(&self, query: GetBookmarkQuery) -> Result<Self::Response, Self::Error> {
        let bookmark = self
            .bookmark_query_repository
            .query_by_id(query.id, query.user_id)
            .await?
            .ok_or(BookmarkError::NotFound(query.id))?;

        Ok(bookmark)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetBookmarkError {
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
