use core::str;
use std::{collections::HashMap, io::BufReader};

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use futures::stream::BoxStream;
use url::Url;

use super::{
    Error, ExtractedFeed, Feed, FeedFindParams, FeedRepository, FeedScrapedData, FeedScraper,
    ProcessedFeed, ScraperError,
};

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

    pub async fn detect_feeds(&self, mut data: FeedDetect) -> Result<DetectedResponse, Error> {
        let host = data.url.host_str().unwrap();

        match self.plugins.get(host) {
            Some(plugin) => {
                let feed = plugin.scrape(&mut data.url).await?;

                let id = self
                    .repository
                    .upsert_feed(FeedScrapedData {
                        url: data.url,
                        feed: feed.clone(),
                        link_to_users: false,
                    })
                    .await?;

                let mut feeds = self
                    .repository
                    .find_feeds(FeedFindParams {
                        id: Some(id),
                        ..Default::default()
                    })
                    .await?;

                Ok(DetectedResponse::Processed(feeds.swap_remove(0)))
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

                        let id = self
                            .repository
                            .upsert_feed(FeedScrapedData {
                                url: data.url,
                                feed: feed.clone(),
                                link_to_users: false,
                            })
                            .await?;

                        let mut feeds = self
                            .repository
                            .find_feeds(FeedFindParams {
                                id: Some(id),
                                ..Default::default()
                            })
                            .await?;

                        Ok(DetectedResponse::Processed(feeds.swap_remove(0)))
                    }
                    _ => Err(Error::Scraper(ScraperError::Unsupported)),
                }
            }
        }
    }

    pub async fn scrape_feed(&self, mut data: FeedScrape) -> Result<(), Error> {
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
            .upsert_feed(FeedScrapedData {
                url: data.url,
                feed,
                link_to_users: true,
            })
            .await?;

        Ok(())
    }

    pub async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        self.repository.stream_feed_urls().await
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum DetectedResponse {
    Detected(Vec<FeedDetected>),
    Processed(Feed),
}

#[derive(Debug, Clone)]
pub struct FeedScrape {
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
