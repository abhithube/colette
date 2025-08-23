use colette_queue::JobProducer;
use tokio::sync::Mutex;

use crate::{
    Handler,
    auth::UserId,
    bookmark::{BookmarkError, BookmarkId, BookmarkRepository},
    common::RepositoryError,
    job::JobRepository,
};

#[derive(Debug, Clone)]
pub struct DeleteBookmarkCommand {
    pub id: BookmarkId,
    pub user_id: UserId,
}

pub struct DeleteBookmarkHandler<BR: BookmarkRepository, JR: JobRepository, JP: JobProducer> {
    bookmark_repository: BR,
    job_repository: JR,
    archive_thumbnail_producer: Mutex<JP>,
}

impl<BR: BookmarkRepository, JR: JobRepository, JP: JobProducer> DeleteBookmarkHandler<BR, JR, JP> {
    pub fn new(
        bookmark_repository: BR,
        job_repository: JR,
        archive_thumbnail_producer: JP,
    ) -> Self {
        Self {
            bookmark_repository,
            job_repository,
            archive_thumbnail_producer: Mutex::new(archive_thumbnail_producer),
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository, JR: JobRepository, JP: JobProducer> Handler<DeleteBookmarkCommand>
    for DeleteBookmarkHandler<BR, JR, JP>
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

        // let data = serde_json::to_value(&ArchiveThumbnailJobData {
        //     operation: ThumbnailOperation::Delete,
        //     archived_path: bookmark.archived_path,
        //     bookmark_id: bookmark.id,
        // })?;

        // let job_id = self
        //     .job_repository
        //     .insert(JobInsertParams {
        //         job_type: "archive_thumbnail".into(),
        //         data,
        //         group_identifier: None,
        //     })
        //     .await?;

        // let mut producer = self.archive_thumbnail_producer.lock().await;

        // producer.push(job_id.as_inner()).await?;

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
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
