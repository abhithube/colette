use chrono::{DateTime, Utc};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use url::Url;

use crate::{
    Bookmark, Handler,
    auth::UserId,
    bookmark::{BookmarkAuthor, BookmarkError, BookmarkRepository, BookmarkTitle},
    common::RepositoryError,
    job::JobRepository,
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

pub struct CreateBookmarkHandler<BR: BookmarkRepository, JR: JobRepository, JP: JobProducer> {
    bookmark_repository: BR,
    job_repository: JR,
    archive_thumbnail_producer: Mutex<JP>,
}

impl<BR: BookmarkRepository, JR: JobRepository, JP: JobProducer> CreateBookmarkHandler<BR, JR, JP> {
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
impl<BR: BookmarkRepository, JR: JobRepository, JP: JobProducer> Handler<CreateBookmarkCommand>
    for CreateBookmarkHandler<BR, JR, JP>
{
    type Response = Bookmark;
    type Error = CreateBookmarkError;

    async fn handle(&self, cmd: CreateBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let title = BookmarkTitle::new(cmd.title)?;
        let author = cmd.author.map(BookmarkAuthor::new).transpose()?;

        let bookmark = Bookmark::new(
            cmd.url.clone(),
            title,
            cmd.thumbnail_url,
            cmd.published_at,
            author,
            cmd.user_id,
        );

        self.bookmark_repository
            .save(&bookmark)
            .await
            .map_err(|e| match e {
                RepositoryError::Duplicate => {
                    CreateBookmarkError::Bookmark(BookmarkError::Conflict(cmd.url))
                }
                _ => CreateBookmarkError::Repository(e),
            })?;

        // if let Some(thumbnail_url) = cmd.thumbnail_url {
        //     let data = serde_json::to_value(&ArchiveThumbnailJobData {
        //         operation: ThumbnailOperation::Upload(thumbnail_url),
        //         archived_path: None,
        //         bookmark_id: id,
        //     })?;

        //     let job_id = self
        //         .job_repository
        //         .insert(JobInsertParams {
        //             job_type: "archive_thumbnail".into(),
        //             data,
        //             group_identifier: None,
        //         })
        //         .await?;

        //     let mut producer = self.archive_thumbnail_producer.lock().await;

        //     producer.push(job_id.as_inner()).await?;
        // }

        Ok(bookmark)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBookmarkError {
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
