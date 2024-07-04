use crate::common::FindOneParams;

use super::{Error, Feed, ProcessedFeed};
use async_trait::async_trait;

#[async_trait]
pub trait FeedsRepository {
    async fn find_many(&self, params: FeedFindManyParams<'_>) -> Result<Vec<Feed>, Error>;

    async fn find_one(&self, params: FindOneParams<'_>) -> Result<Feed, Error>;

    async fn create(&self, data: FeedCreateData<'_>) -> Result<Feed, Error>;
}

pub struct FeedFindManyParams<'a> {
    pub profile_id: &'a str,
}

pub struct FeedCreateData<'a> {
    pub url: &'a str,
    pub feed: ProcessedFeed,
    pub profile_id: &'a str,
}
