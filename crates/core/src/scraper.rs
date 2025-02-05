use std::sync::Arc;

pub use colette_scraper::feed::ProcessedFeed;
use colette_scraper::{
    bookmark::{BookmarkScraper, ProcessedBookmark},
    feed::FeedScraper,
};
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct FeedCreate {
    pub url: Url,
}

#[derive(Clone, Debug)]
pub struct BookmarkCreate {
    pub url: Url,
    pub user_id: Uuid,
}

pub struct ScraperService {
    repository: Box<dyn ScraperRepository>,
    feed_scraper: Arc<dyn FeedScraper>,
    bookmark_scraper: Arc<dyn BookmarkScraper>,
}

impl ScraperService {
    pub fn new(
        repository: impl ScraperRepository,
        feed_scraper: Arc<dyn FeedScraper>,
        bookmark_scraper: Arc<dyn BookmarkScraper>,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            feed_scraper,
            bookmark_scraper,
        }
    }

    pub async fn scrape_feed(&self, mut data: FeedCreate) -> Result<(), Error> {
        let feed = self.feed_scraper.scrape(&mut data.url).await?;

        self.repository
            .save_feed(SaveFeedData {
                url: data.url.to_string(),
                feed,
            })
            .await
    }

    pub async fn scrape_bookmark(&self, mut data: BookmarkCreate) -> Result<(), Error> {
        let bookmark = self.bookmark_scraper.scrape(&mut data.url).await?;

        self.repository
            .save_bookmark(SaveBookmarkData {
                url: data.url.to_string(),
                bookmark,
                user_id: data.user_id,
            })
            .await
    }
}

#[async_trait::async_trait]
pub trait ScraperRepository: Send + Sync + 'static {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error>;

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct SaveFeedData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Clone, Debug)]
pub struct SaveBookmarkData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub user_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
