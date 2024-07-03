use super::{Error, Feed, ProcessedFeed};
use async_trait::async_trait;

#[async_trait]
pub trait FeedsRepository {
    async fn create(&self, data: FeedCreateData) -> Result<Feed, Error>;
}

pub struct FeedCreateData {
    pub url: String,
    pub feed: ProcessedFeed,
    pub profile_id: String,
}
