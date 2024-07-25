use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use http::Response;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{FindManyParams, FindOneParams, Paginated, SendableStream, Session},
    utils::{
        backup::{self, BackupManager},
        scraper::{
            self, DownloaderPlugin, ExtractorPlugin, ExtractorQuery, PostprocessorPlugin, Scraper,
        },
    },
};

#[derive(Clone, Debug)]
pub struct Feed {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct CreateFeed {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct UpdateFeed {
    pub title: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DetectFeeds {
    pub url: String,
}

#[derive(Clone, Debug)]
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

#[async_trait::async_trait]
pub trait FeedsRepository: Send + Sync {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Feed>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Feed, Error>;

    async fn create(&self, data: FeedsCreateData) -> Result<Feed, Error>;

    async fn update(&self, params: FindOneParams, data: FeedsUpdateData) -> Result<Feed, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;

    fn iterate(&self) -> SendableStream<Result<(i64, String), Error>>;

    async fn cleanup(&self) -> Result<(), Error>;
}

pub trait Detector: Send + Sync {
    fn detect(
        &self,
        url: &str,
        resp: Response<String>,
    ) -> Result<Vec<String>, scraper::ExtractError>;
}

pub enum DetectorPlugin<'a> {
    Value(Vec<ExtractorQuery<'a>>),
    Impl(Arc<dyn Detector>),
}

pub trait FeedScraper: Scraper<ProcessedFeed> {
    fn detect(&self, url: &mut String) -> Result<Vec<String>, scraper::Error>;
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

    pub async fn list(&self, session: Session) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repo
            .find_many(FindManyParams {
                profile_id: session.profile_id,
            })
            .await?;

        let paginated = Paginated::<Feed> {
            has_more: false,
            data: feeds,
        };

        Ok(paginated)
    }

    pub async fn get(&self, id: Uuid, session: Session) -> Result<Feed, Error> {
        let feed = self
            .repo
            .find_one(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(feed)
    }

    pub async fn create(&self, mut data: CreateFeed, session: Session) -> Result<Feed, Error> {
        let scraped = self.scraper.scrape(&mut data.url)?;

        let feed = self
            .repo
            .create(FeedsCreateData {
                url: data.url,
                feed: scraped,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(feed)
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: UpdateFeed,
        session: Session,
    ) -> Result<Feed, Error> {
        let feed = self
            .repo
            .update(
                FindOneParams {
                    id,
                    profile_id: session.profile_id,
                },
                data.into(),
            )
            .await?;

        Ok(feed)
    }

    pub async fn delete(&self, id: Uuid, session: Session) -> Result<(), Error> {
        self.repo
            .delete(FindOneParams {
                id,
                profile_id: session.profile_id,
            })
            .await?;

        Ok(())
    }

    pub async fn detect(&self, mut data: DetectFeeds) -> Result<Paginated<DetectedFeed>, Error> {
        let urls = self.scraper.detect(&mut data.url)?;

        let mut feeds: Vec<DetectedFeed> = vec![];

        for mut url in urls.into_iter() {
            let feed = self.scraper.scrape(&mut url)?;
            feeds.push(DetectedFeed {
                url,
                title: feed.title,
            })
        }

        let paginated = Paginated::<DetectedFeed> {
            has_more: false,
            data: feeds,
        };

        Ok(paginated)
    }

    pub async fn import(&self, data: ImportFeeds, session: Session) -> Result<(), Error> {
        for feed in self.opml.import(&data.raw)? {
            self.create(
                CreateFeed {
                    url: feed.xml_url.to_string(),
                },
                session.clone(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn export(&self, session: Session) -> Result<String, Error> {
        let feeds = self.list(session).await?;

        let data = feeds
            .data
            .into_iter()
            .filter_map(|e| {
                Some(BackupFeed {
                    title: e.title,
                    xml_url: e.url.and_then(|e| Url::parse(&e).ok())?,
                    html_url: Url::parse(&e.link).ok(),
                })
            })
            .collect::<Vec<_>>();

        let raw = self.opml.export(data)?;

        Ok(raw)
    }
}

#[derive(Clone, Debug)]
pub struct FeedsCreateData {
    pub url: String,
    pub feed: ProcessedFeed,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FeedsUpdateData {
    pub custom_title: Option<String>,
}

impl From<UpdateFeed> for FeedsUpdateData {
    fn from(value: UpdateFeed) -> Self {
        Self {
            custom_title: value.title,
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
