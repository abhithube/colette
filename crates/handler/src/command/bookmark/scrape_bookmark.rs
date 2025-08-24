use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_scraper::bookmark::{BookmarkError, BookmarkScraper};
use url::Url;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ScrapeBookmarkCommand {
    pub url: Url,
}

pub struct ScrapeBookmarkHandler<HC: HttpClient> {
    bookmark_scraper: Arc<BookmarkScraper<HC>>,
}

impl<HC: HttpClient> ScrapeBookmarkHandler<HC> {
    pub fn new(bookmark_scraper: Arc<BookmarkScraper<HC>>) -> Self {
        Self { bookmark_scraper }
    }
}

#[async_trait::async_trait]
impl<HC: HttpClient> Handler<ScrapeBookmarkCommand> for ScrapeBookmarkHandler<HC> {
    type Response = BookmarkScraped;
    type Error = ScrapeBookmarkError;

    async fn handle(&self, mut cmd: ScrapeBookmarkCommand) -> Result<Self::Response, Self::Error> {
        let processed = self.bookmark_scraper.scrape(&mut cmd.url).await?;

        Ok(BookmarkScraped {
            link: cmd.url,
            title: processed.title,
            thumbnail_url: processed.thumbnail,
            published_at: processed.published,
            author: processed.author,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BookmarkScraped {
    pub link: Url,
    pub title: String,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScrapeBookmarkError {
    #[error(transparent)]
    Scraper(#[from] BookmarkError),
}
