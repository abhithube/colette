use crate::{
    Handler,
    bookmark::{Bookmark, BookmarkError, BookmarkFindParams, BookmarkId, BookmarkRepository},
    common::RepositoryError,
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct GetBookmarkQuery {
    pub id: BookmarkId,
    pub with_tags: bool,
    pub user_id: UserId,
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
        bookmark.authorize(query.user_id)?;

        Ok(bookmark)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetBookmarkError {
    #[error("bookmark not found with ID: {0}")]
    NotFound(BookmarkId),

    #[error(transparent)]
    Core(#[from] BookmarkError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
