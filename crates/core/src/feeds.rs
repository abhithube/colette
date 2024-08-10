use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use futures::stream::BoxStream;
use http::Response;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{FindOneParams, Paginated, Session},
    tags::CreateTag,
    utils::{
        backup::{self, BackupManager},
        scraper::{
            self, DownloaderPlugin, ExtractorPlugin, ExtractorQuery, PostprocessorPlugin, Scraper,
        },
    },
    Tag,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub original_title: String,
    pub url: Option<String>,
    pub tags: Vec<Tag>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CreateFeed {
    pub url: Url,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct UpdateFeed {
    pub title: Option<Option<String>>,
    pub tags: Option<Vec<CreateTag>>,
}

#[derive(Clone, Debug)]
pub struct ListFeedsParams {
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct DetectFeeds {
    pub url: Url,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct DetectedFeed {
    pub url: String,
    pub title: String,
}

pub struct ImportFeeds {
    pub raw: String,
}

#[derive(Clone, Debug)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selector: &'a str,
    pub entry_link_queries: Vec<ExtractorQuery<'a>>,
    pub entry_title_queries: Vec<ExtractorQuery<'a>>,
    pub entry_published_queries: Vec<ExtractorQuery<'a>>,
    pub entry_description_queries: Vec<ExtractorQuery<'a>>,
    pub entry_author_queries: Vec<ExtractorQuery<'a>>,
    pub entry_thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

#[derive(Clone, Debug)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedEntry>,
}

#[derive(Clone, Debug)]
pub struct ExtractedEntry {
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
    pub entries: Vec<ProcessedEntry>,
}

#[derive(Clone, Debug)]
pub struct ProcessedEntry {
    pub link: Url,
    pub title: String,
    pub published: Option<DateTime<Utc>>,
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

#[derive(Clone, Debug)]
pub struct StreamFeed {
    pub id: i32,
    pub url: String,
}

#[async_trait::async_trait]
pub trait FeedsRepository: Send + Sync {
    async fn find_many_feeds(&self, params: FeedsFindManyParams) -> Result<Vec<Feed>, Error>;

    async fn find_one_feed(&self, params: FindOneParams) -> Result<Feed, Error>;

    async fn create_feed(&self, data: FeedsCreateData) -> Result<Feed, Error>;

    async fn update_feed(
        &self,
        params: FindOneParams,
        data: FeedsUpdateData,
    ) -> Result<Feed, Error>;

    async fn delete_feed(&self, params: FindOneParams) -> Result<(), Error>;

    async fn stream_feeds(&self) -> Result<BoxStream<Result<StreamFeed, Error>>, Error>;

    async fn cleanup_feeds(&self) -> Result<(), Error>;
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

pub struct FeedPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin<()>>,
    pub detectors: HashMap<&'static str, DetectorPlugin<'a>>,
    pub extractors: HashMap<&'static str, ExtractorPlugin<FeedExtractorOptions<'a>, ExtractedFeed>>,
    pub postprocessors:
        HashMap<&'static str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>>,
}

pub struct FeedsService {
    repo: Arc<dyn FeedsRepository>,
    scraper: Arc<dyn FeedScraper>,
    opml: Arc<dyn BackupManager<T = Vec<BackupFeed>>>,
}

impl FeedsService {
    pub fn new(
        repo: Arc<dyn FeedsRepository>,
        scraper: Arc<dyn FeedScraper>,
        opml: Arc<dyn BackupManager<T = Vec<BackupFeed>>>,
    ) -> Self {
        Self {
            repo,
            scraper,
            opml,
        }
    }

    pub async fn list(
        &self,
        params: ListFeedsParams,
        session: Session,
    ) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repo
            .find_many_feeds(FeedsFindManyParams {
                profile_id: session.profile_id,
                tags: params.tags,
            })
            .await?;

        Ok(Paginated::<Feed> {
            has_more: false,
            data: feeds,
        })
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Feed, Error> {
        self.repo
            .find_one_feed(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn create(&self, mut data: CreateFeed, session: Session) -> Result<Feed, Error> {
        let scraped = self.scraper.scrape(&mut data.url)?;

        self.repo
            .create_feed(FeedsCreateData {
                url: data.url.into(),
                feed: scraped,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateFeed,
        session: Session,
    ) -> Result<Feed, Error> {
        self.repo
            .update_feed(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete_feed(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await
    }

    pub async fn detect(&self, mut data: DetectFeeds) -> Result<Paginated<DetectedFeed>, Error> {
        let urls = self.scraper.detect(&mut data.url)?;

        let mut feeds: Vec<DetectedFeed> = vec![];

        for mut url in urls.into_iter() {
            let feed = self.scraper.scrape(&mut url)?;
            feeds.push(DetectedFeed {
                url: url.into(),
                title: feed.title,
            })
        }

        Ok(Paginated::<DetectedFeed> {
            has_more: false,
            data: feeds,
        })
    }

    pub async fn import(&self, data: ImportFeeds, session: Session) -> Result<(), Error> {
        for feed in self.opml.import(&data.raw)? {
            self.create(CreateFeed { url: feed.xml_url }, session.clone())
                .await?;
        }

        Ok(())
    }

    pub async fn export(&self, session: Session) -> Result<String, Error> {
        let feeds = self.list(ListFeedsParams { tags: None }, session).await?;

        let data = feeds
            .data
            .into_iter()
            .filter_map(|e| {
                Some(BackupFeed {
                    title: e.title.unwrap_or(e.original_title),
                    xml_url: e.url.and_then(|e| Url::parse(&e).ok())?,
                    html_url: Url::parse(&e.link).ok(),
                })
            })
            .collect::<Vec<_>>();

        self.opml.export(data).map_err(|e| e.into())
    }
}

#[derive(Clone, Debug)]
pub struct FeedsFindManyParams {
    pub profile_id: Uuid,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug)]
pub struct FeedsCreateData {
    pub url: String,
    pub feed: ProcessedFeed,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedsUpdateData {
    pub title: Option<String>,
    pub update_title: bool,
    pub tags: Option<Vec<String>>,
}

impl From<UpdateFeed> for FeedsUpdateData {
    fn from(value: UpdateFeed) -> Self {
        let update_title = value.title.is_some();
        Self {
            title: value.title.flatten(),
            update_title,
            tags: value.tags.map(|e| e.into_iter().map(|e| e.title).collect()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Scraper(#[from] scraper::Error),

    #[error(transparent)]
    Backup(#[from] backup::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
