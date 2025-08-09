use std::sync::Arc;

use colette_scraper::bookmark::{BookmarkError, BookmarkScraper};
use url::Url;
use uuid::Uuid;

use super::{BookmarkInsertParams, BookmarkRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct RefreshBookmarkCommand {
    pub url: Url,
    pub user_id: Uuid,
}

pub struct RefreshBookmarkHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    bookmark_scraper: Arc<BookmarkScraper>,
}

impl RefreshBookmarkHandler {
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        bookmark_scraper: Arc<BookmarkScraper>,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            bookmark_scraper,
        }
    }
}

#[async_trait::async_trait]
impl Handler<RefreshBookmarkCommand> for RefreshBookmarkHandler {
    type Response = ();
    type Error = RefreshBookmarkError;

    async fn handle(&self, mut cmd: RefreshBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let processed = self.bookmark_scraper.scrape(&mut cmd.url).await?;

        self.bookmark_repository
            .insert(BookmarkInsertParams {
                link: cmd.url,
                title: processed.title,
                thumbnail_url: processed.thumbnail,
                published_at: processed.published,
                author: processed.author,
                user_id: cmd.user_id,
                upsert: true,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshBookmarkError {
    #[error(transparent)]
    Scraper(#[from] BookmarkError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
