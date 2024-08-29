use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use colette_backup::{
    opml::{Opml, OpmlBody, OpmlOutline, OpmlOutlineType},
    BackupManager,
};
use futures::stream::BoxStream;
use http::Response;
use url::Url;
use uuid::Uuid;

use crate::{
    common::{Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable},
    scraper::{
        self, DownloaderPlugin, ExtractorPlugin, ExtractorQuery, PostprocessorPlugin, Scraper,
    },
    tag::TagCreate,
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedCreate {
    pub url: Url,
    pub folder_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedUpdate {
    pub title: Option<Option<NonEmptyString>>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<TagCreate>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedListQuery {
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FeedDetect {
    pub url: Url,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedDetected {
    pub url: String,
    pub title: String,
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

pub struct FeedService {
    repository: Arc<dyn FeedRepository>,
    scraper: Arc<dyn FeedScraper>,
    opml_manager: Arc<dyn BackupManager<T = Opml>>,
}

impl FeedService {
    pub fn new(
        repository: Arc<dyn FeedRepository>,
        scraper: Arc<dyn FeedScraper>,
        opml_manager: Arc<dyn BackupManager<T = Opml>>,
    ) -> Self {
        Self {
            repository,
            scraper,
            opml_manager,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Feed>, Error> {
        self.repository
            .list(profile_id, None, None, Some(query.into()))
            .await
    }

    pub async fn get_feed(&self, id: Uuid, profile_id: Uuid) -> Result<Feed, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create_feed(&self, mut data: FeedCreate, profile_id: Uuid) -> Result<Feed, Error> {
        let url = data.url.to_string();
        let folder_id = Some(data.folder_id);

        let result = self
            .repository
            .create(FeedCreateData {
                url: url.clone(),
                feed: None,
                folder_id,
                profile_id,
            })
            .await;

        match result {
            Ok(data) => Ok(data),
            Err(Error::Conflict(_)) => {
                let scraped = self.scraper.scrape(&mut data.url)?;

                self.repository
                    .create(FeedCreateData {
                        url,
                        feed: Some(scraped),
                        folder_id,
                        profile_id,
                    })
                    .await
            }
            e => e,
        }
    }

    pub async fn update_feed(
        &self,
        id: Uuid,
        data: FeedUpdate,
        profile_id: Uuid,
    ) -> Result<Feed, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }

    pub async fn delete_feed(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }

    pub async fn detect_feeds(
        &self,
        mut data: FeedDetect,
    ) -> Result<Paginated<FeedDetected>, Error> {
        let urls = self.scraper.detect(&mut data.url)?;

        let mut feeds: Vec<FeedDetected> = vec![];

        for mut url in urls.into_iter() {
            let feed = self.scraper.scrape(&mut url)?;
            let url = url.to_string();

            feeds.push(FeedDetected {
                url: url.clone(),
                title: feed.title.clone(),
            });

            self.repository.cache(FeedCacheData { url, feed }).await?;
        }

        Ok(Paginated::<FeedDetected> {
            data: feeds,
            cursor: None,
        })
    }

    pub async fn import_feeds(&self, raw: String, profile_id: Uuid) -> Result<(), Error> {
        let opml = self.opml_manager.import(&raw)?;

        for outline in opml.body.outlines {
            if let Some(xml_url) = outline.xml_url {
                self.create_feed(
                    FeedCreate {
                        url: xml_url,
                        folder_id: None,
                    },
                    profile_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn export_feeds(&self, profile_id: Uuid) -> Result<Bytes, Error> {
        let feeds = self
            .list_feeds(FeedListQuery::default(), profile_id)
            .await?;

        let data = feeds
            .data
            .iter()
            .cloned()
            .map(|e| OpmlOutline {
                outline_type: Some(OpmlOutlineType::default()),
                text: e.title.clone().unwrap_or(e.original_title.clone()),
                title: Some(e.title.unwrap_or(e.original_title)),
                xml_url: e.url.and_then(|e| Url::parse(&e).ok()),
                html_url: Url::parse(&e.link).ok(),
                children: None,
            })
            .collect::<Vec<_>>();

        let opml = Opml {
            body: OpmlBody { outlines: data },
            ..Default::default()
        };

        self.opml_manager.export(opml).map_err(|e| e.into())
    }
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

impl From<FeedListQuery> for FeedFindManyFilters {
    fn from(value: FeedListQuery) -> Self {
        Self { tags: value.tags }
    }
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

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title.map(|e| e.map(String::from)),
            folder_id: value.folder_id,
            tags: value
                .tags
                .map(|e| e.into_iter().map(|e| e.title.into()).collect()),
        }
    }
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
    Backup(#[from] colette_backup::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
