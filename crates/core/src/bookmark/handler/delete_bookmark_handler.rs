use colette_queue::JobProducer;
use tokio::sync::Mutex;

use crate::{
    Handler,
    bookmark::{
        ArchiveThumbnailJobData, BookmarkError, BookmarkId, BookmarkRepository, ThumbnailOperation,
    },
    common::RepositoryError,
    job::{JobInsertParams, JobRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct DeleteBookmarkCommand {
    pub id: BookmarkId,
    pub user_id: UserId,
}

pub struct DeleteBookmarkHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    job_repository: Box<dyn JobRepository>,
    archive_thumbnail_producer: Box<Mutex<dyn JobProducer>>,
}

impl DeleteBookmarkHandler {
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        job_repository: impl JobRepository,
        archive_thumbnail_producer: impl JobProducer,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            job_repository: Box::new(job_repository),
            archive_thumbnail_producer: Box::new(Mutex::new(archive_thumbnail_producer)),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DeleteBookmarkCommand> for DeleteBookmarkHandler {
    type Response = ();
    type Error = DeleteBookmarkError;

    async fn handle(&self, cmd: DeleteBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let bookmark = self
            .bookmark_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| DeleteBookmarkError::NotFound(cmd.id))?;
        bookmark.authorize(cmd.user_id)?;

        self.bookmark_repository.delete_by_id(cmd.id).await?;

        let data = serde_json::to_value(&ArchiveThumbnailJobData {
            operation: ThumbnailOperation::Delete,
            archived_path: bookmark.archived_path,
            bookmark_id: bookmark.id,
        })?;

        let job_id = self
            .job_repository
            .insert(JobInsertParams {
                job_type: "archive_thumbnail".into(),
                data,
                group_identifier: None,
            })
            .await?;

        let mut producer = self.archive_thumbnail_producer.lock().await;

        producer.push(job_id.as_inner()).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteBookmarkError {
    #[error("bookmark not found with ID: {0}")]
    NotFound(BookmarkId),

    #[error(transparent)]
    Core(#[from] BookmarkError),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
