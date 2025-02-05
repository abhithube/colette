use core::str;
use std::io::{BufRead, BufReader};

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_feed::Feed;
pub use extractor::{FeedExtractor, FeedExtractorOptions};
pub use registry::FeedPluginRegistry;
use reqwest::Client;
use url::Url;

use crate::{Error, PostprocessorError};

mod atom;
mod extractor;
mod registry;
mod rss;

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

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
pub struct DetectedFeed {
    pub url: String,
    pub title: String,
}

impl From<colette_meta::rss::Feed> for DetectedFeed {
    fn from(value: colette_meta::rss::Feed) -> Self {
        Self {
            url: value.href,
            title: value.title,
        }
    }
}

pub enum DetectorResponse {
    Detected(Vec<DetectedFeed>),
    Processed(ProcessedFeed),
}

impl TryFrom<ExtractedFeed> for ProcessedFeed {
    type Error = PostprocessorError;

    fn try_from(value: ExtractedFeed) -> Result<Self, Self::Error> {
        let Some(Ok(link)) = value.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessorError::Link);
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError::Title);
        };
        if title.is_empty() {
            return Err(PostprocessorError::Title);
        }

        let mut entries: Vec<ProcessedFeedEntry> = Vec::new();

        for entry in value.entries.into_iter() {
            entries.push(entry.try_into()?);
        }

        let feed = Self {
            link,
            title: title.trim().to_owned(),
            entries,
        };

        Ok(feed)
    }
}

impl TryFrom<ExtractedFeedEntry> for ProcessedFeedEntry {
    type Error = PostprocessorError;

    fn try_from(value: ExtractedFeedEntry) -> Result<Self, Self::Error> {
        let Some(Ok(link)) = value.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessorError::Link);
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError::Title);
        };
        if title.is_empty() {
            return Err(PostprocessorError::Title);
        }

        let Some(published) = value.published.as_ref().and_then(|e| {
            DateTime::parse_from_rfc3339(e.trim())
                .ok()
                .or(DateTime::parse_from_rfc2822(e).ok())
                .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                .map(|f| f.to_utc())
        }) else {
            return Err(PostprocessorError::Published);
        };
        let thumbnail = value
            .thumbnail
            .as_ref()
            .and_then(|e| Url::parse(e.trim()).ok());

        let entry = Self {
            link,
            title: title.trim().to_owned(),
            published,
            description: value.description.and_then(|e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e.trim().to_owned())
                }
            }),
            author: value.author.and_then(|e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e.trim().to_owned())
                }
            }),
            thumbnail,
        };

        Ok(entry)
    }
}

impl From<Feed> for ExtractedFeed {
    fn from(value: Feed) -> Self {
        match value {
            Feed::Atom(atom) => atom.into(),
            Feed::Rss(rss) => rss.into(),
        }
    }
}

#[async_trait::async_trait]
pub trait FeedScraper: Send + Sync + 'static {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error>;
}

#[async_trait::async_trait]
pub trait FeedDetector: Send + Sync {
    async fn detect(&self, mut url: Url) -> Result<DetectorResponse, Error>;
}

#[derive(Clone)]
pub struct DefaultFeedScraper {
    client: Client,
}

impl DefaultFeedScraper {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl FeedScraper for DefaultFeedScraper {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let resp = self.client.get(url.as_str()).send().await?;
        let body = resp.bytes().await?;

        let feed = colette_feed::from_reader(BufReader::new(body.reader()))
            .map(ExtractedFeed::from)
            .map_err(|e| Error::Parse(e.into()))?;

        Ok(feed.try_into()?)
    }
}

#[derive(Clone)]
pub struct DefaultFeedDetector {
    client: Client,
}

impl DefaultFeedDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl FeedDetector for DefaultFeedDetector {
    async fn detect(&self, url: Url) -> Result<DetectorResponse, Error> {
        let resp = self.client.get(url.as_str()).send().await?;
        let body = resp.bytes().await?;

        let mut reader = BufReader::new(body.reader());
        let buffer = reader.fill_buf()?;

        let raw = str::from_utf8(buffer)?;

        match raw {
            raw if raw.contains("<!DOCTYPE html") => {
                let metadata =
                    colette_meta::parse_metadata(reader).map_err(|e| Error::Parse(e.into()))?;

                let feeds = metadata.feeds.into_iter().map(DetectedFeed::from).collect();

                Ok(DetectorResponse::Detected(feeds))
            }
            raw if raw.contains("<?xml") => {
                let feed = colette_feed::from_reader(BufReader::new(reader))
                    .map(ExtractedFeed::from)
                    .map_err(|e| Error::Parse(e.into()))?;

                Ok(DetectorResponse::Processed(feed.try_into()?))
            }
            _ => Err(Error::Unsupported),
        }
    }
}
