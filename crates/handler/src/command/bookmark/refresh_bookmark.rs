use std::sync::Arc;

use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::BookmarkRepository;
use colette_http::HttpClient;
use colette_scraper::bookmark::{BookmarkError, BookmarkScraper};
use url::Url;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct RefreshBookmarkCommand {
    pub url: Url,
    pub user_id: UserId,
}

pub struct RefreshBookmarkHandler<BR: BookmarkRepository, HC: HttpClient> {
    bookmark_repository: BR,
    bookmark_scraper: Arc<BookmarkScraper<HC>>,
}

impl<BR: BookmarkRepository, HC: HttpClient> RefreshBookmarkHandler<BR, HC> {
    pub fn new(bookmark_repository: BR, bookmark_scraper: Arc<BookmarkScraper<HC>>) -> Self {
        Self {
            bookmark_repository,
            bookmark_scraper,
        }
    }
}

impl<BR: BookmarkRepository, HC: HttpClient> Handler<RefreshBookmarkCommand>
    for RefreshBookmarkHandler<BR, HC>
{
    type Response = ();
    type Error = RefreshBookmarkError;

    async fn handle(&self, mut cmd: RefreshBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let processed = self.bookmark_scraper.scrape(&mut cmd.url).await?;

        // self.bookmark_repository
        //     .insert(BookmarkInsertParams {
        //         link: cmd.url,
        //         title: processed.title,
        //         thumbnail_url: processed.thumbnail,
        //         published_at: processed.published,
        //         author: processed.author,
        //         user_id: cmd.user_id,
        //         upsert: true,
        //     })
        //     .await?;

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
