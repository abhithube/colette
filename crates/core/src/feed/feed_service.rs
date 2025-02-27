use core::str;
use std::{collections::HashMap, io::BufReader};

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use futures::stream::BoxStream;
use url::Url;
use uuid::Uuid;

use super::{
    Error, ExtractedFeed, Feed, FeedScraper, ScraperError,
    feed_repository::{
        FeedCreateData, FeedFindParams, FeedRepository, FeedScrapedData, FeedUpdateData,
    },
    feed_scraper::ProcessedFeed,
};
use crate::common::{IdParams, Paginated};

pub struct FeedService {
    repository: Box<dyn FeedRepository>,
    client: Box<dyn HttpClient>,
    plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
}

impl FeedService {
    pub fn new(
        repository: impl FeedRepository,
        client: impl HttpClient,
        plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            client: Box::new(client),
            plugins,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Feed>, Error> {
        let feeds = self
            .repository
            .find(FeedFindParams {
                tags: query.tags,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: feeds,
            cursor: None,
        })
    }

    pub async fn get_feed(&self, id: Uuid, user_id: Uuid) -> Result<Feed, Error> {
        let mut feeds = self
            .repository
            .find(FeedFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(feeds.swap_remove(0))
    }

    pub async fn create_feed(&self, data: FeedCreate, user_id: Uuid) -> Result<Feed, Error> {
        let id = self
            .repository
            .create(FeedCreateData {
                url: data.url,
                title: data.title,
                tags: data.tags,
                user_id,
            })
            .await?;

        self.get_feed(id, user_id).await
    }

    pub async fn update_feed(
        &self,
        id: Uuid,
        data: FeedUpdate,
        user_id: Uuid,
    ) -> Result<Feed, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_feed(id, user_id).await
    }

    pub async fn delete_feed(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }

    pub async fn detect_feeds(&self, mut data: FeedDetect) -> Result<DetectedResponse, Error> {
        let host = data.url.host_str().unwrap();

        match self.plugins.get(host) {
            Some(plugin) => {
                let feed = plugin.scrape(&mut data.url).await?;

                self.repository
                    .save_scraped(FeedScrapedData {
                        url: data.url,
                        feed: feed.clone(),
                        link_to_users: false,
                    })
                    .await?;

                Ok(DetectedResponse::Processed(feed))
            }
            None => {
                let body = self.client.get(&data.url).await?;

                let mut reader = BufReader::new(body.reader());

                let raw = str::from_utf8(reader.peek(14)?)?;
                match raw {
                    raw if raw.contains("<!DOCTYPE html") => {
                        let metadata = colette_meta::parse_metadata(reader)
                            .map_err(|e| ScraperError::Parse(e.into()))?;

                        let feeds = metadata
                            .feeds
                            .into_iter()
                            .map(FeedDetected::from)
                            .collect::<Vec<_>>();

                        Ok(DetectedResponse::Detected(feeds))
                    }
                    raw if raw.contains("<?xml") => {
                        let feed = colette_feed::from_reader(reader)
                            .map(ExtractedFeed::from)
                            .map_err(|e| ScraperError::Parse(e.into()))?;

                        let feed =
                            ProcessedFeed::try_from(feed).map_err(|e| Error::Scraper(e.into()))?;

                        self.repository
                            .save_scraped(FeedScrapedData {
                                url: data.url,
                                feed: feed.clone(),
                                link_to_users: false,
                            })
                            .await?;

                        Ok(DetectedResponse::Processed(feed))
                    }
                    _ => Err(Error::Scraper(ScraperError::Unsupported)),
                }
            }
        }
    }

    pub async fn scrape_and_persist_feed(&self, mut data: FeedPersist) -> Result<(), Error> {
        let host = data.url.host_str().unwrap();

        let feed = match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(&mut data.url).await,
            None => {
                let body = self.client.get(&data.url).await?;
                let feed = colette_feed::from_reader(BufReader::new(body.reader()))
                    .map(ExtractedFeed::from)
                    .map_err(|e| ScraperError::Parse(e.into()))?;

                Ok(feed.try_into().map_err(ScraperError::Postprocess)?)
            }
        }?;

        self.repository
            .save_scraped(FeedScrapedData {
                url: data.url,
                feed,
                link_to_users: true,
            })
            .await
    }

    pub async fn stream(&self) -> Result<BoxStream<Result<String, Error>>, Error> {
        self.repository.stream_urls().await
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeedListQuery {
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct FeedCreate {
    pub url: Url,
    pub title: String,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Default)]
pub struct FeedUpdate {
    pub title: Option<String>,
    pub tags: Option<Vec<Uuid>>,
}

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title,
            tags: value.tags,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeedDetect {
    pub url: Url,
}

#[derive(Debug, Clone)]
pub struct FeedDetected {
    pub url: Url,
    pub title: String,
}

impl From<colette_meta::rss::Feed> for FeedDetected {
    fn from(value: colette_meta::rss::Feed) -> Self {
        Self {
            url: value.href.parse().unwrap(),
            title: value.title,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DetectedResponse {
    Detected(Vec<FeedDetected>),
    Processed(ProcessedFeed),
}

#[derive(Debug, Clone)]
pub struct FeedPersist {
    pub url: Url,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScrapeFeedJob {
    pub url: Url,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RefreshFeedsJob(pub DateTime<Utc>);

impl From<DateTime<Utc>> for RefreshFeedsJob {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}
