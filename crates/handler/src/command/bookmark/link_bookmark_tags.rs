use colette_core::{
    auth::UserId,
    bookmark::{BookmarkError, BookmarkId, BookmarkRepository},
    common::RepositoryError,
    tag::TagId,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct LinkBookmarkTagsCommand {
    pub id: BookmarkId,
    pub tag_ids: Vec<TagId>,
    pub user_id: UserId,
}

pub struct LinkBookmarkTagsHandler<BR: BookmarkRepository> {
    bookmark_repository: BR,
}

impl<BR: BookmarkRepository> LinkBookmarkTagsHandler<BR> {
    pub fn new(bookmark_repository: BR) -> Self {
        Self {
            bookmark_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository> Handler<LinkBookmarkTagsCommand> for LinkBookmarkTagsHandler<BR> {
    type Response = ();
    type Error = LinkBookmarkTagsError;

    async fn handle(&self, cmd: LinkBookmarkTagsCommand) -> Result<Self::Response, Self::Error> {
        let mut bookmark = self
            .bookmark_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or_else(|| LinkBookmarkTagsError::Bookmark(BookmarkError::NotFound(cmd.id)))?;

        bookmark.set_tags(cmd.tag_ids)?;

        self.bookmark_repository.save(&bookmark).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkBookmarkTagsError {
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
