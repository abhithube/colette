use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_scraper::bookmark::{BookmarkError, BookmarkScraper};
use url::Url;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ScrapeBookmarkCommand {
    pub url: Url,
}

pub struct ScrapeBookmarkHandler {
    bookmark_scraper: Arc<BookmarkScraper>,
}

impl ScrapeBookmarkHandler {
    pub fn new(bookmark_scraper: Arc<BookmarkScraper>) -> Self {
        Self { bookmark_scraper }
    }
}

#[async_trait::async_trait]
impl Handler<ScrapeBookmarkCommand> for ScrapeBookmarkHandler {
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
