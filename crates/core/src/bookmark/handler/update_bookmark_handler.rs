use chrono::{DateTime, Utc};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use url::Url;

use crate::{
    Bookmark, Handler,
    auth::UserId,
    bookmark::{BookmarkAuthor, BookmarkError, BookmarkId, BookmarkRepository, BookmarkTitle},
    common::RepositoryError,
    job::JobRepository,
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
    type Response = Bookmark;
    type Error = UpdateBookmarkError;

    async fn handle(&self, cmd: UpdateBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let mut bookmark = self
            .bookmark_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or_else(|| UpdateBookmarkError::Bookmark(BookmarkError::NotFound(cmd.id)))?;

        if let Some(title) = cmd.title.map(BookmarkTitle::new).transpose()? {
            bookmark.set_title(title);
        }
        if let Some(thumbnail_url) = cmd.thumbnail_url {
            if let Some(thumbnail_url) = thumbnail_url {
                bookmark.set_thumbnail_url(thumbnail_url);
            } else {
                bookmark.remove_thumbnail_url();
            }
        }
        if let Some(published_at) = cmd.published_at {
            if let Some(published_at) = published_at {
                bookmark.set_published_at(published_at);
            } else {
                bookmark.remove_published_at();
            }
        }
        if let Some(author) = cmd.author {
            if let Some(author) = author.map(BookmarkAuthor::new).transpose()? {
                bookmark.set_author(author);
            } else {
                bookmark.remove_author();
            }
        }

        self.bookmark_repository.save(&bookmark).await?;

        // if let Some(thumbnail_url) = new_thumbnail
        //     && thumbnail_url != bookmark.thumbnail_url
        // {
        //     let data = serde_json::to_value(&ArchiveThumbnailJobData {
        //         operation: if let Some(thumbnail_url) = thumbnail_url {
        //             ThumbnailOperation::Upload(thumbnail_url)
        //         } else {
        //             ThumbnailOperation::Delete
        //         },
        //         archived_path: bookmark.archived_path.clone(),
        //         bookmark_id: bookmark.id,
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
pub enum UpdateBookmarkError {
    #[error(transparent)]
    Bookmark(#[from] BookmarkError),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
