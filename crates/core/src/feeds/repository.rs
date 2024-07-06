use async_trait::async_trait;

use super::{Error, Feed, ProcessedFeed};
use crate::common::FindOneParams;

#[async_trait]
pub trait FeedsRepository {
    async fn find_many(&self, params: FeedFindManyParams) -> Result<Vec<Feed>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Feed, Error>;

    async fn create(&self, data: FeedCreateData) -> Result<Feed, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

pub struct FeedFindManyParams {
    pub profile_id: String,
}

pub struct FeedCreateData {
    pub url: String,
    pub feed: ProcessedFeed,
    pub profile_id: String,
}
