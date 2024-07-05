use async_trait::async_trait;

use super::{Error, Feed, ProcessedFeed};
use crate::common::FindOneParams;

#[async_trait]
pub trait FeedsRepository {
    async fn find_many(&self, params: FeedFindManyParams<'_>) -> Result<Vec<Feed>, Error>;

    async fn find_one(&self, params: FindOneParams<'_>) -> Result<Feed, Error>;

    async fn create(&self, data: FeedCreateData<'_>) -> Result<Feed, Error>;

    async fn delete(&self, params: FindOneParams<'_>) -> Result<(), Error>;
}

pub struct FeedFindManyParams<'a> {
    pub profile_id: &'a str,
}

pub struct FeedCreateData<'a> {
    pub url: &'a str,
    pub feed: ProcessedFeed,
    pub profile_id: &'a str,
}
