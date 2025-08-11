use chrono::{DateTime, Utc};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use url::Url;

use crate::{
    Handler,
    bookmark::{
        ArchiveThumbnailJobData, BookmarkError, BookmarkId, BookmarkRepository,
        BookmarkUpdateParams, ThumbnailOperation,
    },
    common::RepositoryError,
    job::{JobInsertParams, JobRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct UpdateBookmarkCommand {
    pub id: BookmarkId,
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub user_id: UserId,
}

pub struct UpdateBookmarkHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    job_repository: Box<dyn JobRepository>,
    archive_thumbnail_producer: Box<Mutex<dyn JobProducer>>,
}

impl UpdateBookmarkHandler {
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
impl Handler<UpdateBookmarkCommand> for UpdateBookmarkHandler {
    type Response = ();
    type Error = UpdateBookmarkError;

    async fn handle(&self, cmd: UpdateBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let bookmark = self
            .bookmark_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| UpdateBookmarkError::NotFound(cmd.id))?;
        bookmark.authorize(cmd.user_id)?;

        let new_thumbnail = cmd.thumbnail_url.clone();

        self.bookmark_repository
            .update(BookmarkUpdateParams {
                id: cmd.id,
                title: cmd.title,
                thumbnail_url: cmd.thumbnail_url,
                published_at: cmd.published_at,
                author: cmd.author,
            })
            .await?;

        if let Some(thumbnail_url) = new_thumbnail
            && thumbnail_url != bookmark.thumbnail_url
        {
            let data = serde_json::to_value(&ArchiveThumbnailJobData {
                operation: if let Some(thumbnail_url) = thumbnail_url {
                    ThumbnailOperation::Upload(thumbnail_url)
                } else {
                    ThumbnailOperation::Delete
                },
                archived_path: bookmark.archived_path.clone(),
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
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateBookmarkError {
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
