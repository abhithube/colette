use crate::{
    Handler,
    auth::UserId,
    bookmark::{BookmarkError, BookmarkId, BookmarkRepository},
    common::RepositoryError,
    tag::TagId,
};

#[derive(Debug, Clone)]
pub struct LinkBookmarkTagsCommand {
    pub id: BookmarkId,
    pub tag_ids: Vec<TagId>,
    pub user_id: UserId,
}

pub struct LinkBookmarkTagsHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
}

impl LinkBookmarkTagsHandler {
    pub fn new(bookmark_repository: impl BookmarkRepository) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<LinkBookmarkTagsCommand> for LinkBookmarkTagsHandler {
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
