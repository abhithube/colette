use colette_queue::JobProducer;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::BookmarkRepository;
use crate::{
    Handler, RepositoryError,
    bookmark::{ArchiveThumbnailJobData, ThumbnailOperation},
    job::{JobInsertParams, JobRepository},
};

#[derive(Debug, Clone)]
pub struct DeleteBookmarkCommand {
    pub id: Uuid,
    pub user_id: Uuid,
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
        let Some(bookmark) = self.bookmark_repository.find_by_id(cmd.id).await? else {
            return Err(DeleteBookmarkError::NotFound(cmd.id));
        };
        if bookmark.user_id != cmd.user_id {
            return Err(DeleteBookmarkError::Forbidden(cmd.id));
        }

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

        producer.push(job_id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteBookmarkError {
    #[error("bookmark not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access bookmark with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
