use uuid::Uuid;

use super::{BookmarkLinkTagParams, BookmarkRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct LinkBookmarkTagsCommand {
    pub id: Uuid,
    pub tag_ids: Vec<Uuid>,
    pub user_id: Uuid,
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
        let Some(bookmark) = self.bookmark_repository.find_by_id(cmd.id).await? else {
            return Err(LinkBookmarkTagsError::NotFound(cmd.id));
        };
        if bookmark.user_id != cmd.user_id {
            return Err(LinkBookmarkTagsError::Forbidden(cmd.id));
        }

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
    NotFound(Uuid),

    #[error("not authorized to access bookmark with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
