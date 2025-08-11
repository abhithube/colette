use chrono::{DateTime, Utc};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use url::Url;

use crate::{
    Handler, RepositoryError,
    bookmark::{
        ArchiveThumbnailJobData, BookmarkId, BookmarkInsertParams, BookmarkRepository,
        ThumbnailOperation,
    },
    job::{JobInsertParams, JobRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct CreateBookmarkCommand {
    pub url: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub user_id: UserId,
}

pub struct CreateBookmarkHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    job_repository: Box<dyn JobRepository>,
    archive_thumbnail_producer: Box<Mutex<dyn JobProducer>>,
}

impl CreateBookmarkHandler {
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
impl Handler<CreateBookmarkCommand> for CreateBookmarkHandler {
    type Response = BookmarkCreated;
    type Error = CreateBookmarkError;

    async fn handle(&self, cmd: CreateBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .bookmark_repository
            .insert(BookmarkInsertParams {
                link: cmd.url.clone(),
                title: cmd.title,
                thumbnail_url: cmd.thumbnail_url.clone(),
                published_at: cmd.published_at,
                author: cmd.author,
                user_id: cmd.user_id,
                upsert: false,
            })
            .await
            .map_err(|e| match e {
                RepositoryError::Duplicate => CreateBookmarkError::Conflict(cmd.url),
                _ => CreateBookmarkError::Repository(e),
            })?;

        if let Some(thumbnail_url) = cmd.thumbnail_url {
            let data = serde_json::to_value(&ArchiveThumbnailJobData {
                operation: ThumbnailOperation::Upload(thumbnail_url),
                archived_path: None,
                bookmark_id: id,
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

        Ok(BookmarkCreated { id })
    }
}

#[derive(Debug, Clone)]
pub struct BookmarkCreated {
    pub id: BookmarkId,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBookmarkError {
    #[error("bookmark already exists with URL: {0}")]
    Conflict(Url),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
