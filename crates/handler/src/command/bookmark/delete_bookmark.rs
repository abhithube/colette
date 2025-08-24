use colette_core::{
    auth::UserId,
    bookmark::{BookmarkError, BookmarkId, BookmarkRepository},
    common::RepositoryError,
};
use colette_queue::JobProducer;
use tokio::sync::Mutex;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DeleteBookmarkCommand {
    pub id: BookmarkId,
    pub user_id: UserId,
}

pub struct DeleteBookmarkHandler<BR: BookmarkRepository, JP: JobProducer> {
    bookmark_repository: BR,
    archive_thumbnail_producer: Mutex<JP>,
}

impl<BR: BookmarkRepository, JP: JobProducer> DeleteBookmarkHandler<BR, JP> {
    pub fn new(bookmark_repository: BR, archive_thumbnail_producer: JP) -> Self {
        Self {
            bookmark_repository,
            archive_thumbnail_producer: Mutex::new(archive_thumbnail_producer),
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository, JP: JobProducer> Handler<DeleteBookmarkCommand>
    for DeleteBookmarkHandler<BR, JP>
{
    type Response = ();
    type Error = DeleteBookmarkError;

    async fn handle(&self, cmd: DeleteBookmarkCommand) -> Result<Self::Response, Self::Error> {
        self.bookmark_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    DeleteBookmarkError::Bookmark(BookmarkError::NotFound(cmd.id))
                }
                _ => DeleteBookmarkError::Repository(e),
            })?;

        // let data = ArchiveThumbnailJobData {
        //     operation: ThumbnailOperation::Delete,
        //     archived_path: bookmark.archived_path,
        //     bookmark_id: cmd.id,
        // };
        // let job = Job::create("archive_thumbnail", data)?;

        // let mut producer = self.archive_thumbnail_producer.lock().await;

        // producer.push(job).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteBookmarkError {
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
