use super::{model::CreateFeedDto, Error, FeedFindManyParams, FeedsRepository, ProcessedFeed};
use crate::{
    common::{FindOneParams, Paginated, Session},
    feeds::FeedCreateData,
    scraper::Scraper,
    Feed,
};

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

    pub async fn list(&self, session: Session) -> Result<Paginated<Feed>, Error> {
        let params = FeedFindManyParams {
            profile_id: session.profile_id.as_str(),
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
            id: id.as_str(),
            profile_id: session.profile_id.as_str(),
        };
        let feed = self.feeds_repo.find_one(params).await?;

        Ok(feed)
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
