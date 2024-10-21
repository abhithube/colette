use std::sync::Arc;

use colette_scraper::FeedScraper;
pub use colette_scraper::ProcessedFeed;
use url::Url;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedCreate {
    pub url: Url,
}

pub struct ScraperService {
    repository: Arc<dyn ScraperRepository>,
    scraper: Arc<dyn FeedScraper>,
}

impl ScraperService {
    pub fn new(repository: Arc<dyn ScraperRepository>, scraper: Arc<dyn FeedScraper>) -> Self {
        Self {
            repository,
            scraper,
        }
    }

    pub async fn scrape_feed(&self, mut data: FeedCreate) -> Result<(), Error> {
        let feed = self.scraper.scrape(&mut data.url)?;

        self.repository
            .save_feed(SaveFeedData {
                url: data.url.to_string(),
                feed,
            })
            .await
    }
}

#[async_trait::async_trait]
pub trait ScraperRepository: Send + Sync {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct SaveFeedData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
