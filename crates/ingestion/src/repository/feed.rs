use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_scraper::feed::ProcessedFeedEntry;
use url::Url;

use crate::{Feed, FeedId};

pub trait FeedRepository: Sync {
    fn find(
        &self,
        params: FeedFindParams,
    ) -> impl Future<Output = Result<Vec<Feed>, RepositoryError>> + Send;

    fn find_by_source_url(
        &self,
        source_url: Url,
    ) -> impl Future<Output = Result<Option<Feed>, RepositoryError>> + Send;

    fn find_outdated(
        &self,
        params: FeedFindOutdatedParams,
    ) -> impl Future<Output = Result<Vec<Feed>, RepositoryError>> + Send;

    fn upsert(
        &self,
        params: FeedUpsertParams,
    ) -> impl Future<Output = Result<FeedId, RepositoryError>> + Send;

    fn mark_as_failed(
        &self,
        source_url: Url,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

#[derive(Debug, Clone, Default)]
pub struct FeedFindParams {
    pub id: Option<FeedId>,
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
