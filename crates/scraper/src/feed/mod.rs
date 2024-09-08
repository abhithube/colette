use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
pub use detector::*;
use feed_rs::{
    model::{Entry, Feed, Link},
    parser::Builder,
};
use http::{Request, Response};
use scraper::{Html, Selector};
use url::Url;

use crate::{
    utils::{ExtractorQuery, TextSelector},
    DownloaderError, DownloaderPlugin, ExtractorError, PostprocessorError, Scraper,
};

mod detector;

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

#[derive(Clone, Debug, Default)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selectors: Vec<Selector>,
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
        if title.is_empty() {
            return Err(PostprocessorError(anyhow!("could not process feed title")));
        }

        let mut entries: Vec<ProcessedFeedEntry> = vec![];

        for entry in value.entries.into_iter() {
            let Some(Ok(link)) = entry.link.as_ref().map(|e| Url::parse(e)) else {
                return Err(PostprocessorError(anyhow!("could not process entry link")));
            };
            let Some(title) = entry.title else {
                return Err(PostprocessorError(anyhow!("could not process entry title")));
            };
            if title.is_empty() {
                return Err(PostprocessorError(anyhow!("could not process entry title")));
            }

            let Some(published) = entry.published.as_ref().and_then(|e| {
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
            let thumbnail = entry
                .thumbnail
                .as_ref()
                .and_then(|e| Url::parse(e.trim()).ok());

            entries.push(ProcessedFeedEntry {
                link,
                title: title.trim().to_owned(),
                published,
                description: entry.description.and_then(|e| {
                    if e.is_empty() {
                        None
                    } else {
                        Some(e.trim().to_owned())
                    }
                }),
                author: entry.author.and_then(|e| {
                    if e.is_empty() {
                        None
                    } else {
                        Some(e.trim().to_owned())
                    }
                }),
                thumbnail,
            });
        }

        let feed = ProcessedFeed {
            link,
            title: title.trim().to_owned(),
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

pub struct FeedPlugin<'a> {
    pub downloader: DownloaderPlugin,
    pub detector: Option<FeedDetectorPlugin<'a>>,
    pub extractor: Option<FeedExtractorOptions<'a>>,
    pub postprocessor: FeedPostprocessorPlugin,
}

impl Default for FeedPlugin<'_> {
    fn default() -> Self {
        Self {
            downloader: |url| {
                Request::get(url.as_str())
                    .body(())
                    .map(|e| e.into_parts().0)
                    .map_err(|e| DownloaderError(e.into()))
            },
            detector: None,
            extractor: None,
            postprocessor: |_url, _extracted| Ok(()),
        }
    }
}

pub struct DefaultFeedScraper<'a> {
    registry: HashMap<&'static str, FeedPlugin<'a>>,
    default_plugin: FeedPlugin<'a>,
    default_detector: Box<dyn FeedDetector>,
}

impl<'a> DefaultFeedScraper<'a> {
    pub fn new(registry: HashMap<&'static str, FeedPlugin<'a>>) -> Self {
        Self {
            registry,
            default_plugin: FeedPlugin::default(),
            default_detector: Box::new(DefaultFeedDetector::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl Scraper<ProcessedFeed> for DefaultFeedScraper<'_> {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;
        let plugin = self.registry.get(host).unwrap_or(&self.default_plugin);

        let parts = (plugin.downloader)(url)?;
        let req: ureq::Request = parts.into();
        let resp = tokio::task::spawn(async move { req.call() })
            .await
            .map_err(|e| DownloaderError(e.into()))?
            .map_err(|e| DownloaderError(e.into()))?;

        let mut extracted = if let Some(options) = &plugin.extractor {
            let raw = resp.into_string().map_err(|e| DownloaderError(e.into()))?;
            let html = Html::parse_document(&raw);

            let entries = options
                .feed_entries_selectors
                .iter()
                .find_map(|e| {
                    let entries = html
                        .select(e)
                        .map(|element| ExtractedFeedEntry {
                            link: element.select_text(&options.feed_entry_link_queries),
                            title: element.select_text(&options.feed_entry_title_queries),
                            published: element.select_text(&options.feed_entry_published_queries),
                            description: element
                                .select_text(&options.feed_entry_description_queries),
                            author: element.select_text(&options.feed_entry_author_queries),
                            thumbnail: element.select_text(&options.feed_entry_thumbnail_queries),
                        })
                        .collect::<Vec<_>>();

                    if entries.is_empty() {
                        None
                    } else {
                        Some(entries)
                    }
                })
                .unwrap_or_default();

            let mut feed = ExtractedFeed {
                link: html.select_text(&options.feed_link_queries),
                title: html.select_text(&options.feed_title_queries),
                entries,
            };
            if feed.link.is_none() {
                feed.link = Some(url.to_string());
            }

            Ok(feed)
        } else {
            let parser = Builder::new()
                .timestamp_parser(|e| {
                    DateTime::parse_from_rfc3339(e.trim())
                        .ok()
                        .or(DateTime::parse_from_rfc2822(e).ok())
                        .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                        .map(|f| f.to_utc())
                })
                .build();

            parser
                .parse(resp.into_reader())
                .map(ExtractedFeed::from)
                .map_err(|e| ExtractorError(e.into()))
        }?;

        (plugin.postprocessor)(url, &mut extracted)?;
        let processed = extracted.try_into()?;

        Ok(processed)
    }
}

impl FeedScraper for DefaultFeedScraper<'_> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;
        let plugin = self.registry.get(host).unwrap_or(&self.default_plugin);

        let parts = (plugin.downloader)(url)?;
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
