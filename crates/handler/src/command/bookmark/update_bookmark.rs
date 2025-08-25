use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{
    Bookmark, BookmarkAuthor, BookmarkError, BookmarkId, BookmarkRepository, BookmarkTitle,
};
use colette_queue::JobProducer;
use tokio::sync::Mutex;
use url::Url;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct UpdateBookmarkCommand {
    pub id: BookmarkId,
    pub title: Option<String>,
    pub thumbnail_url: Option<Option<Url>>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub author: Option<Option<String>>,
    pub user_id: UserId,
}

pub struct UpdateBookmarkHandler<BR: BookmarkRepository, JP: JobProducer> {
    bookmark_repository: BR,
    archive_thumbnail_producer: Mutex<JP>,
}

impl<BR: BookmarkRepository, JP: JobProducer> UpdateBookmarkHandler<BR, JP> {
    pub fn new(bookmark_repository: BR, archive_thumbnail_producer: JP) -> Self {
        Self {
            bookmark_repository,
            archive_thumbnail_producer: Mutex::new(archive_thumbnail_producer),
        }
    }
}

impl<BR: BookmarkRepository, JP: JobProducer> Handler<UpdateBookmarkCommand>
    for UpdateBookmarkHandler<BR, JP>
{
    type Response = Bookmark;
    type Error = UpdateBookmarkError;

    async fn handle(&self, cmd: UpdateBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let mut bookmark = self
            .bookmark_repository
            .find_by_id(cmd.id, cmd.user_id)
            .await?
            .ok_or(BookmarkError::NotFound(cmd.id.as_inner()))?;

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

        // if let Some(thumbnail_url) = cmd.thumbnail_url
        //     && thumbnail_url != bookmark.thumbnail_url().cloned()
        // {
        //     let data = ArchiveThumbnailJobData {
        //         operation: if let Some(thumbnail_url) = thumbnail_url {
        //             ThumbnailOperation::Upload(thumbnail_url)
        //         } else {
        //             ThumbnailOperation::Delete
        //         },
        //         archived_path: bookmark.archived_path.clone(),
        //         bookmark_id: bookmark.id(),
        //     };
        //     let job = Job::create("archive_thumbnail", data)?;

        //     let mut producer = self.archive_thumbnail_producer.lock().await;

        //     producer.push(job).await?;
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
    Repository(#[from] RepositoryError),
}
