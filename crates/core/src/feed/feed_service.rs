use core::str;
use std::{collections::HashMap, io::BufReader};

use bytes::Buf;
use colette_http::HttpClient;
use futures::stream::BoxStream;
use url::Url;

use super::{Error, ExtractedFeed, Feed, FeedRepository, FeedScraper, ProcessedFeed, ScraperError};

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
        if let Some(feed) = self.repository.find_by_source_url(data.url.clone()).await? {
            return Ok(DetectedResponse::Processed(feed));
        }

        let host = data.url.host_str().unwrap();

        match self.plugins.get(host) {
            Some(plugin) => {
                let processed = plugin.scrape(&mut data.url).await?;

                let mut feed: Feed = (data.url, processed).into();
                feed.is_custom = true;

                self.repository.save(&feed).await?;

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

                        let processed =
                            ProcessedFeed::try_from(feed).map_err(|e| Error::Scraper(e.into()))?;

                        let feed: Feed = (data.url, processed).into();

                        self.repository.save(&feed).await?;

                        Ok(DetectedResponse::Processed(feed))
                    }
                    _ => Err(Error::Scraper(ScraperError::Unsupported)),
                }
            }
        }
    }

    pub async fn scrape_feed(&self, mut data: FeedScrape) -> Result<(), Error> {
        let host = data.url.host_str().unwrap();

        let feed = match self.plugins.get(host) {
            Some(plugin) => {
                let processed = plugin.scrape(&mut data.url).await?;

                let mut feed: Feed = (data.url, processed).into();
                feed.is_custom = true;

                feed
            }
            None => {
                let body = self.client.get(&data.url).await?;
                let extracted = colette_feed::from_reader(BufReader::new(body.reader()))
                    .map(ExtractedFeed::from)
                    .map_err(|e| ScraperError::Parse(e.into()))?;

                let processed = extracted.try_into().map_err(ScraperError::Postprocess)?;

                (data.url, processed).into()
            }
        };

        self.repository.save(&feed).await?;

        Ok(())
    }

    pub async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        self.repository.stream().await
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
pub struct ScrapeFeedJobData {
    pub url: Url,
}
