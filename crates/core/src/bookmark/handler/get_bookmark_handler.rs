use crate::{
    Handler,
    auth::UserId,
    bookmark::{BookmarkDto, BookmarkError, BookmarkFindParams, BookmarkId, BookmarkRepository},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct GetBookmarkQuery {
    pub id: BookmarkId,
    pub user_id: UserId,
}

pub struct GetBookmarkHandler<BR: BookmarkRepository> {
    bookmark_repository: BR,
}

impl<BR: BookmarkRepository> GetBookmarkHandler<BR> {
    pub fn new(bookmark_repository: BR) -> Self {
        Self {
            bookmark_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository> Handler<GetBookmarkQuery> for GetBookmarkHandler<BR> {
    type Response = BookmarkDto;
    type Error = GetBookmarkError;

    async fn handle(&self, query: GetBookmarkQuery) -> Result<Self::Response, Self::Error> {
        let mut bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id: query.user_id,
                id: Some(query.id),
                filter: None,
                tags: None,
                cursor: None,
                limit: None,
            })
            .await?;
        if bookmarks.is_empty() {
            return Err(GetBookmarkError::Bookmark(BookmarkError::NotFound(
                query.id,
            )));
        }

        let bookmark = bookmarks.swap_remove(0);

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
