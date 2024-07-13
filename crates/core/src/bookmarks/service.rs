use std::sync::Arc;

use super::{model::ListBookmarksParams, BookmarkFindManyParams, BookmarksRepository, Error};
use crate::{
    common::{FindOneParams, Paginated, Session, PAGINATION_LIMIT},
    Bookmark,
};

pub struct BookmarksService {
    repo: Arc<dyn BookmarksRepository + Send + Sync>,
}

impl BookmarksService {
    pub fn new(repo: Arc<dyn BookmarksRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        params: ListBookmarksParams,
        session: Session,
    ) -> Result<Paginated<Bookmark>, Error> {
        let params = BookmarkFindManyParams {
            profile_id: session.profile_id,
            limit: (PAGINATION_LIMIT + 1) as i64,
            published_at: params.published_at,
            should_filter: params.collection_id.is_none() && params.is_default.is_some_and(|e| e),
            collection_id: params.collection_id,
        };
        let bookmarks = self.repo.find_many(params).await?;

        let paginated = Paginated::<Bookmark> {
            has_more: bookmarks.len() > PAGINATION_LIMIT,
            data: bookmarks.into_iter().take(PAGINATION_LIMIT).collect(),
        };

        Ok(paginated)
    }

    pub async fn delete(&self, id: String, session: Session) -> Result<(), Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        self.repo.delete(params).await?;

        Ok(())
    }
}
