use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use futures::stream::BoxStream;
use http::Response;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable},
    scraper::{
        self, DownloaderPlugin, ExtractorPlugin, ExtractorQuery, PostprocessorPlugin, Scraper,
    },
    Tag,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub original_title: String,
    pub url: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selector: &'a str,
    pub feed_entry_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_published_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_description_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_author_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedFeedEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedFeedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProcessedFeed {
    pub link: Url,
    pub title: String,
    pub entries: Vec<ProcessedFeedEntry>,
}

#[derive(Clone, Debug)]
pub struct ProcessedFeedEntry {
    pub link: Url,
    pub title: String,
    pub published: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<Url>,
}

#[derive(Clone, Debug)]
pub struct BackupFeed {
    pub title: String,
    pub xml_url: Url,
    pub html_url: Option<Url>,
}

#[async_trait::async_trait]
pub trait FeedRepository:
    Findable<Params = IdParams, Output = Result<Feed, Error>>
    + Creatable<Data = FeedCreateData, Output = Result<Feed, Error>>
    + Updatable<Params = IdParams, Data = FeedUpdateData, Output = Result<Feed, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<FeedFindManyFilters>,
    ) -> Result<Paginated<Feed>, Error>;

    async fn cache(&self, data: FeedCacheData) -> Result<(), Error>;

    async fn stream(&self) -> Result<BoxStream<Result<(i32, String), Error>>, Error>;

    async fn cleanup(&self) -> Result<(), Error>;
}

pub trait Detector: Send + Sync {
    fn detect(&self, url: &Url, resp: Response<String>) -> Result<Vec<Url>, scraper::ExtractError>;
}

pub enum DetectorPlugin<'a> {
    Value(Vec<ExtractorQuery<'a>>),
    Impl(Arc<dyn Detector>),
}

pub trait FeedScraper: Scraper<ProcessedFeed> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, scraper::Error>;
}

#[derive(Default)]
pub struct FeedPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin<()>>,
    pub detectors: HashMap<&'static str, DetectorPlugin<'a>>,
    pub extractors: HashMap<&'static str, ExtractorPlugin<FeedExtractorOptions<'a>, ExtractedFeed>>,
    pub postprocessors:
        HashMap<&'static str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedFindManyFilters {
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedCreateData {
    pub url: String,
    pub feed: Option<ProcessedFeed>,
    pub folder_id: Option<Option<Uuid>>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedCacheData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Clone, Debug, Default)]
pub struct FeedUpdateData {
    pub title: Option<Option<String>>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with id: {0}")]
    NotFound(Uuid),

    #[error("feed not cached with URL: {0}")]
    Conflict(String),

    #[error(transparent)]
    Scraper(#[from] scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
