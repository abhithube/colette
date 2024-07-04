use super::{model::CreateFeedDto, Error, FeedsRepository, ProcessedFeed};
use crate::{common::Session, feeds::FeedCreateData, scraper::Scraper, Feed};

pub struct FeedsService {
    feeds_repo: Box<dyn FeedsRepository + Send + Sync>,
    scraper: Box<dyn Scraper<ProcessedFeed> + Send + Sync>,
}

impl FeedsService {
    pub fn new(
        feeds_repo: Box<dyn FeedsRepository + Send + Sync>,
        scraper: Box<dyn Scraper<ProcessedFeed> + Send + Sync>,
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
