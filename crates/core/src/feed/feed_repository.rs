use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use super::{Cursor, Error, Feed, ProcessedFeed};
use crate::common::IdParams;

#[async_trait::async_trait]
pub trait FeedRepository: Send + Sync + 'static {
    async fn find_feeds(&self, params: FeedFindParams) -> Result<Vec<Feed>, Error>;

    async fn create_feed(&self, data: FeedCreateData) -> Result<Uuid, Error>;

    async fn update_feed(&self, params: IdParams, data: FeedUpdateData) -> Result<(), Error>;

    async fn delete_feed(&self, params: IdParams) -> Result<(), Error>;

    async fn save_scraped(&self, data: FeedScrapedData) -> Result<(), Error>;

    async fn stream_urls(&self) -> Result<BoxStream<Result<String, Error>>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct FeedCreateData {
    pub url: Url,
    pub title: String,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct FeedUpdateData {
    pub title: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct FeedScrapedData {
    pub url: Url,
    pub feed: ProcessedFeed,
    pub link_to_users: bool,
}
