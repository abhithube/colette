use super::{model::CreateFeedDto, Error, FeedsRepository, ProcessedFeed};
use crate::{common::Session, feeds::FeedCreateData, scraper::Scraper, Feed};
use std::sync::Arc;

pub struct FeedsService {
    feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
    scraper: Arc<dyn for<'a> Scraper<ProcessedFeed<'a>> + Send + Sync>,
}

impl FeedsService {
    pub fn new(
        feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
        scraper: Arc<dyn for<'a> Scraper<ProcessedFeed<'a>> + Send + Sync>,
    ) -> Self {
        Self {
            feeds_repo,
            scraper,
        }
    }

    pub async fn create(&self, dto: CreateFeedDto, session: Session) -> Result<Feed, Error> {
        let url = dto.url.as_str();
        let scraped = self
            .scraper
            .scrape(url)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let data = FeedCreateData {
            url,
            feed: scraped,
            profile_id: session.profile_id.as_str(),
        };
        let feed = self.feeds_repo.create(data).await?;

        Ok(feed)
    }
}
