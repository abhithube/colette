use std::sync::Arc;

pub use colette_scraper::ProcessedFeed;
use colette_scraper::{BookmarkScraper, FeedScraper, ProcessedBookmark};
use dyn_clone::DynClone;
use url::Url;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedCreate {
    pub url: Url,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookmarkCreate {
    pub url: Url,
}

#[derive(Clone)]
pub struct ScraperService {
    repository: Box<dyn ScraperRepository>,
    feed_scraper: Arc<dyn FeedScraper>,
    bookmark_scraper: Arc<dyn BookmarkScraper>,
}

impl ScraperService {
    pub fn new(
        repository: Box<dyn ScraperRepository>,
        feed_scraper: Arc<dyn FeedScraper>,
        bookmark_scraper: Arc<dyn BookmarkScraper>,
    ) -> Self {
        Self {
            repository,
            feed_scraper,
            bookmark_scraper,
        }
    }

    pub async fn scrape_feed(&self, mut data: FeedCreate) -> Result<(), Error> {
        let feed = self.feed_scraper.scrape(&mut data.url)?;

        self.repository
            .save_feed(SaveFeedData {
                url: data.url.to_string(),
                feed,
            })
            .await
    }

    pub async fn scrape_bookmark(&self, mut data: BookmarkCreate) -> Result<(), Error> {
        let bookmark = self.bookmark_scraper.scrape(&mut data.url)?;

        self.repository
            .save_bookmark(SaveBookmarkData {
                url: data.url.to_string(),
                bookmark,
            })
            .await
    }
}

#[async_trait::async_trait]
pub trait ScraperRepository: Send + Sync + DynClone {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error>;

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error>;
}

dyn_clone::clone_trait_object!(ScraperRepository);

#[derive(Clone, Debug)]
pub struct SaveFeedData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Clone, Debug)]
pub struct SaveBookmarkData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
