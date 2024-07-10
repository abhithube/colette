use std::sync::Arc;

use super::{model::CreateFeed, Error, FeedFindManyParams, FeedsRepository, ProcessedFeed};
use crate::{
    common::{FindOneParams, Paginated, Session},
    feeds::FeedCreateData,
    scraper::Scraper,
    Feed,
};

pub struct FeedsService {
    feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
    scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync>,
}

impl FeedsService {
    pub fn new(
        feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
        scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync>,
    ) -> Self {
        Self {
            feeds_repo,
            scraper,
        }
    }

    pub async fn list(&self, session: Session) -> Result<Paginated<Feed>, Error> {
        let params = FeedFindManyParams {
            profile_id: session.profile_id,
        };
        let feeds = self.feeds_repo.find_many(params).await?;

        let paginated = Paginated::<Feed> {
            has_more: false,
            data: feeds,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: String, session: Session) -> Result<Feed, Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        let feed = self.feeds_repo.find_one(params).await?;

        Ok(feed)
    }

    pub async fn create(&self, data: CreateFeed, session: Session) -> Result<Feed, Error> {
        let scraped = self.scraper.scrape(&data.url).await?;

        let data = FeedCreateData {
            url: data.url,
            feed: scraped,
            profile_id: session.profile_id,
        };
        let feed = self.feeds_repo.create(data).await?;

        Ok(feed)
    }

    pub async fn delete(&self, id: String, session: Session) -> Result<(), Error> {
        let params = FindOneParams {
            id,
            profile_id: session.profile_id,
        };
        self.feeds_repo.delete(params).await?;

        Ok(())
    }
}
