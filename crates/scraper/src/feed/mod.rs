use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
pub use detector::*;
use feed_rs::{
    model::{Entry, Feed, Link},
    parser,
};
use http::Response;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    utils::{ExtractorQuery, TextSelector},
    DownloaderError, DownloaderPlugin, ExtractorError, PostprocessorError, Scraper,
    DEFAULT_DOWNLOADER,
};

mod detector;

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

#[derive(Clone, Debug)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selector: Selector,
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

impl TryFrom<ExtractedFeed> for ProcessedFeed {
    type Error = PostprocessorError;

    fn try_from(value: ExtractedFeed) -> Result<Self, Self::Error> {
        let Some(Ok(link)) = value.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessorError(anyhow!("could not process feed link")));
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError(anyhow!("could not process feed title")));
        };

        let mut entries: Vec<ProcessedFeedEntry> = vec![];

        for e in value.entries.into_iter() {
            let Some(Ok(link)) = e.link.as_ref().map(|e| Url::parse(e)) else {
                return Err(PostprocessorError(anyhow!("could not process entry link")));
            };
            let Some(title) = e.title else {
                return Err(PostprocessorError(anyhow!("could not process entry title")));
            };
            let Some(published) = e.published.as_ref().and_then(|e| {
                DateTime::parse_from_rfc3339(e)
                    .ok()
                    .or(DateTime::parse_from_rfc2822(e).ok())
                    .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                    .map(|f| f.to_utc())
            }) else {
                return Err(PostprocessorError(anyhow!(
                    "could not process entry publish date"
                )));
            };
            let thumbnail = e.thumbnail.as_ref().and_then(|e| Url::parse(e).ok());

            let entry = ProcessedFeedEntry {
                link,
                title,
                published,
                description: e.description,
                author: e.author,
                thumbnail,
            };
            entries.push(entry);
        }

        let feed = ProcessedFeed {
            link,
            title,
            entries,
        };

        Ok(feed)
    }
}

pub type FeedPostprocessorPlugin =
    fn(url: &Url, extracted: &mut ExtractedFeed) -> Result<(), PostprocessorError>;

pub trait FeedScraper: Scraper<ProcessedFeed> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error>;
}

#[derive(Default)]
pub struct FeedPlugin<'a> {
    pub downloader: Option<DownloaderPlugin>,
    pub detector: Option<FeedDetectorPlugin<'a>>,
    pub extractor: Option<FeedExtractorOptions<'a>>,
    pub postprocessor: Option<FeedPostprocessorPlugin>,
}

#[derive(Default)]
pub struct FeedPluginRegistry<'a> {
    pub scrapers: HashMap<&'static str, FeedPlugin<'a>>,
}

pub struct DefaultFeedScraper<'a> {
    registry: FeedPluginRegistry<'a>,
    default_downloader: DownloaderPlugin,
    default_detector: Box<dyn FeedDetector>,
}

impl<'a> DefaultFeedScraper<'a> {
    pub fn new(registry: FeedPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: DEFAULT_DOWNLOADER,
            default_detector: Box::new(DefaultFeedDetector::new(None)),
        }
    }
}

impl Scraper<ProcessedFeed> for DefaultFeedScraper<'_> {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;
        let plugin = self.registry.scrapers.get(host);

        let parts = (self.default_downloader)(url)?;
        let req: ureq::Request = parts.into();
        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
        let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

        let extracted = if let Some(plugin) = plugin {
            if let Some(options) = &plugin.extractor {
                let mut body = resp.into_body();

                let mut bytes: Vec<u8> = vec![];
                body.read(&mut bytes)
                    .map_err(|e| ExtractorError(e.into()))?;

                let raw = String::from_utf8_lossy(&bytes);
                let html = Html::parse_document(&raw);

                let entries = html
                    .select(&options.feed_entries_selector)
                    .map(|element| ExtractedFeedEntry {
                        link: element.select_text(&options.feed_entry_link_queries),
                        title: element.select_text(&options.feed_entry_title_queries),
                        published: element.select_text(&options.feed_entry_published_queries),
                        description: element.select_text(&options.feed_entry_description_queries),
                        author: element.select_text(&options.feed_entry_author_queries),
                        thumbnail: element.select_text(&options.feed_entry_thumbnail_queries),
                    })
                    .collect();

                let feed = ExtractedFeed {
                    link: html.select_text(&options.feed_link_queries),
                    title: html.select_text(&options.feed_title_queries),
                    entries,
                };

                Ok(feed)
            } else {
                return Err(crate::Error::Extract(ExtractorError(anyhow!(""))));
            }
        } else {
            parser::parse(resp.into_body())
                .map(ExtractedFeed::from)
                .map_err(|e| ExtractorError(e.into()))
        }?;

        let processed = extracted.try_into()?;

        Ok(processed)
    }
}

impl FeedScraper for DefaultFeedScraper<'_> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;
        let _plugin = self.registry.scrapers.get(host);

        let parts = (self.default_downloader)(url)?;
        let req: ureq::Request = parts.into();
        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
        let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

        let detected = self.default_detector.detect(url, resp)?;

        Ok(detected)
    }
}

impl From<Feed> for ExtractedFeed {
    fn from(value: Feed) -> Self {
        Self {
            link: parse_atom_link(value.links),
            title: value.title.map(|e| e.content),
            entries: value
                .entries
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<Entry> for ExtractedFeedEntry {
    fn from(mut value: Entry) -> Self {
        let mut title = value.title.map(|e| e.content);
        let mut description = value.summary.map(|e| e.content);
        let mut thumbnail = Option::<String>::None;

        if !value.media.is_empty() {
            let mut media = value.media.swap_remove(0);
            if let Some(t) = media.title.map(|e| e.content) {
                title = Some(t);
            }
            if let Some(d) = media.description.map(|e| e.content) {
                description = Some(d);
            }
            if !media.thumbnails.is_empty() {
                thumbnail = Some(media.thumbnails.swap_remove(0).image.uri);
            } else if !media.content.is_empty() {
                let content = media.content.swap_remove(0);
                if let Some(content_type) = content.content_type {
                    if content_type.ty().as_str() == "image" {
                        if let Some(url) = content.url {
                            thumbnail = Some(url.into())
                        }
                    }
                }
            }
        }

        Self {
            link: parse_atom_link(value.links),
            title,
            published: value.published.map(|e| e.to_rfc3339()),
            description,
            author: Some(
                value
                    .authors
                    .into_iter()
                    .map(|e| e.name)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            thumbnail,
        }
    }
}

fn parse_atom_link(links: Vec<Link>) -> Option<String> {
    links.into_iter().find_map(|l| match l.rel.as_deref() {
        Some("alternate") | None => Some(l.href),
        _ => None,
    })
}
