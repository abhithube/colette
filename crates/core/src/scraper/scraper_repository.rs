use uuid::Uuid;

use super::Error;
use crate::{bookmark::ProcessedBookmark, feed::ProcessedFeed};

#[async_trait::async_trait]
pub trait ScraperRepository: Send + Sync + 'static {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error>;

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct SaveFeedData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Clone, Debug)]
pub struct SaveBookmarkData {
    pub url: String,
    pub bookmark: ProcessedBookmark,
    pub user_id: Uuid,
}
