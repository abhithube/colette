use crate::{
    Handler,
    bookmark::{BookmarkError, BookmarkId, BookmarkLinkTagParams, BookmarkRepository},
    common::RepositoryError,
    tag::TagId,
    auth::UserId,
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
        let bookmark = self
            .bookmark_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| LinkBookmarkTagsError::NotFound(cmd.id))?;
        bookmark.authorize(cmd.user_id)?;

        self.bookmark_repository
            .link_tags(BookmarkLinkTagParams {
                bookmark_id: cmd.id,
                tag_ids: cmd.tag_ids,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkBookmarkTagsError {
    #[error("bookmark not found with ID: {0}")]
    NotFound(BookmarkId),

    #[error(transparent)]
    Core(#[from] BookmarkError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
