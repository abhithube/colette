use chrono::{DateTime, Utc};
use colette_scraper::feed::ProcessedFeedEntry;
use url::Url;
use uuid::Uuid;

use super::{Error, Feed};

#[async_trait::async_trait]
pub trait FeedRepository: Send + Sync + 'static {
    async fn find(&self, params: FeedFindParams) -> Result<Vec<Feed>, Error>;

    async fn find_by_source_url(&self, source_url: Url) -> Result<Option<Feed>, Error>;

    async fn find_outdated(&self, params: FeedFindOutdatedParams) -> Result<Vec<Feed>, Error>;

    async fn upsert(&self, params: FeedUpsertParams) -> Result<Uuid, Error>;

    async fn mark_as_failed(&self, source_url: Url) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct FeedFindParams {
    pub id: Option<Uuid>,
    pub ready_to_refresh: bool,
    pub cursor: Option<Url>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FeedFindOutdatedParams {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FeedUpsertParams {
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    pub refresh_interval_min: u32,
    pub is_custom: bool,
    pub feed_entry_items: Vec<FeedEntryBatchItem>,
}

#[derive(Debug, Clone)]
pub struct FeedEntryBatchItem {
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
}

impl From<ProcessedFeedEntry> for FeedEntryBatchItem {
    fn from(value: ProcessedFeedEntry) -> Self {
        Self {
            link: value.link,
            title: value.title,
            published_at: value.published,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail,
        }
    }
}
