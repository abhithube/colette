use std::io::BufReader;

use anyhow::anyhow;
use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_feed::Feed;
use dyn_clone::DynClone;
use url::Url;

use crate::{Downloader, Error, ExtractorError, PostprocessorError};
pub use extractor::{FeedExtractor, FeedExtractorOptions};
pub use registry::FeedPluginRegistry;

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

impl TryFrom<ExtractedFeed> for ProcessedFeed {
    type Error = PostprocessorError;

    fn try_from(value: ExtractedFeed) -> Result<Self, Self::Error> {
        let Some(Ok(link)) = value.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessorError(anyhow!("could not process feed link")));
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError(anyhow!("could not process feed title")));
        };
        if title.is_empty() {
            return Err(PostprocessorError(anyhow!("could not process feed title")));
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
            return Err(PostprocessorError(anyhow!("could not process value link")));
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError(anyhow!("could not process value title")));
        };
        if title.is_empty() {
            return Err(PostprocessorError(anyhow!("could not process value title")));
        }

        let Some(published) = value.published.as_ref().and_then(|e| {
            DateTime::parse_from_rfc3339(e.trim())
                .ok()
                .or(DateTime::parse_from_rfc2822(e).ok())
                .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                .map(|f| f.to_utc())
        }) else {
            return Err(PostprocessorError(anyhow!(
                "could not process entry publish date"
            )));
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
pub trait FeedScraper: Send + Sync + DynClone {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error>;
}

dyn_clone::clone_trait_object!(FeedScraper);

#[async_trait::async_trait]
pub trait FeedDetector: FeedScraper + Send + Sync {
    async fn detect(&self, mut url: Url) -> Result<Vec<(Url, ProcessedFeed)>, Error>;
}

dyn_clone::clone_trait_object!(FeedDetector);

#[derive(Clone)]
pub struct DefaultFeedScraper {
    downloader: Box<dyn Downloader>,
}

impl DefaultFeedScraper {
    pub fn new(downloader: Box<dyn Downloader>) -> Self {
        Self { downloader }
    }
}

#[async_trait::async_trait]
impl FeedScraper for DefaultFeedScraper {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let body = self.downloader.download(url).await?;

        let feed = colette_feed::from_reader(BufReader::new(body.reader()))
            .map(ExtractedFeed::from)
            .map_err(ExtractorError)?;

        Ok(feed.try_into()?)
    }
}
