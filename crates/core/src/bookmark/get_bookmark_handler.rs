use uuid::Uuid;

use super::{Bookmark, BookmarkFindParams, BookmarkRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct GetBookmarkQuery {
    pub id: Uuid,
    pub with_tags: bool,
    pub user_id: Uuid,
}

pub struct GetBookmarkHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
}

impl GetBookmarkHandler {
    pub fn new(bookmark_repository: impl BookmarkRepository) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetBookmarkQuery> for GetBookmarkHandler {
    type Response = Bookmark;
    type Error = GetBookmarkError;

    async fn handle(&self, query: GetBookmarkQuery) -> Result<Self::Response, Self::Error> {
        let mut bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                id: Some(query.id),
                with_tags: query.with_tags,
                ..Default::default()
            })
            .await?;
        if bookmarks.is_empty() {
            return Err(GetBookmarkError::NotFound(query.id));
        }

        let bookmark = bookmarks.swap_remove(0);
        if bookmark.user_id != query.user_id {
            return Err(GetBookmarkError::Forbidden(query.id));
        }

        Ok(bookmark)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetBookmarkError {
    #[error("bookmark not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access bookmark with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
