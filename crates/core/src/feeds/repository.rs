use super::{Error, Feed, ProcessedFeed};
use async_trait::async_trait;

#[async_trait]
pub trait FeedsRepository {
    async fn create(&self, data: FeedCreateData<'_>) -> Result<Feed, Error>;
}

pub struct FeedCreateData<'a> {
    pub url: &'a str,
    pub feed: ProcessedFeed,
    pub profile_id: &'a str,
}
