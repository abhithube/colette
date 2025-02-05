use std::sync::Arc;

use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};
use url::Url;
use uuid::Uuid;

use super::{
    scraper_repository::{SaveBookmarkData, SaveFeedData, ScraperRepository},
    Error,
};

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

#[derive(Clone, Debug)]
pub struct FeedCreate {
    pub url: Url,
}

#[derive(Clone, Debug)]
pub struct BookmarkCreate {
    pub url: Url,
    pub user_id: Uuid,
}
