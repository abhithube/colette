use std::{collections::HashMap, io::Read};

use chrono::DateTime;
use feed::{ExtractedFeed, ExtractedFeedEntry, FeedExtractorOptions, ProcessedFeed};
use feed_rs::parser;
use http::{
    request::{Builder, Parts},
    Request, Response,
};
use scraper::Html;
use url::Url;

use crate::utils::TextSelector;

pub mod bookmark;
pub mod feed;
pub mod utils;

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

#[async_trait::async_trait]
pub trait Scraper<T>: Send + Sync {
    async fn scrape(&self, url: &mut Url) -> Result<T, Error>;
}

pub trait FeedScraper: Send + Sync {
    fn before_download(&self, url: &mut Url) -> Builder {
        Request::get(url.as_str())
    }

    fn download(
        &self,
        builder: Builder,
    ) -> Result<Response<Box<dyn Read + Send + Sync>>, DownloaderError> {
        let req: ureq::Request = builder
            .try_into()
            .map_err(|e: http::Error| DownloaderError(e.into()))?;

        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        Ok(resp.into())
    }

    fn before_extract(&self) -> Option<FeedExtractorOptions> {
        None
    }

    fn extract(
        &self,
        url: &Url,
        resp: Response<Box<dyn Read + Send + Sync>>,
    ) -> Result<ExtractedFeed, ExtractorError> {
        match self.before_extract() {
            Some(options) => {
                let mut body = resp.into_body();
                let mut raw = String::new();
                body.read_to_string(&mut raw)
                    .map_err(|e| ExtractorError(e.into()))?;

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
                                published: element
                                    .select_text(&options.feed_entry_published_queries),
                                description: element
                                    .select_text(&options.feed_entry_description_queries),
                                author: element.select_text(&options.feed_entry_author_queries),
                                thumbnail: element
                                    .select_text(&options.feed_entry_thumbnail_queries),
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
            }
            None => {
                let parser = parser::Builder::new()
                    .timestamp_parser(|e| {
                        DateTime::parse_from_rfc3339(e.trim())
                            .ok()
                            .or(DateTime::parse_from_rfc2822(e).ok())
                            .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                            .map(|f| f.to_utc())
                    })
                    .build();

                parser
                    .parse(resp.into_body())
                    .map(ExtractedFeed::from)
                    .map_err(|e| ExtractorError(e.into()))
            }
        }
    }

    #[allow(unused_variables)]
    fn before_postprocess(
        &self,
        url: &Url,
        feed: &mut ExtractedFeed,
    ) -> Result<(), PostprocessorError> {
        Ok(())
    }

    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let builder = self.before_download(url);
        let resp = self.download(builder)?;
        let mut feed = self.extract(url, resp)?;
        self.before_postprocess(url, &mut feed)?;

        Ok(feed.try_into()?)
    }
}

#[derive(Default)]
pub struct FeedPluginRegistry {
    plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
}

impl FeedPluginRegistry {
    pub fn new(plugins: HashMap<&'static str, Box<dyn FeedScraper>>) -> Self {
        Self { plugins }
    }
}

impl FeedScraper for FeedPluginRegistry {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let host = url.host_str().ok_or(Error::Parse)?;

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url),
            None => {
                let builder = self.before_download(url);
                let resp = self.download(builder)?;
                let mut feed = self.extract(url, resp)?;
                self.before_postprocess(url, &mut feed)?;

                Ok(feed.try_into()?)
            }
        }
    }
}

pub type DownloaderPlugin = fn(&mut Url) -> Result<Parts, DownloaderError>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Download(#[from] DownloaderError),

    #[error("failed to parse document")]
    Parse,

    #[error(transparent)]
    Extract(#[from] ExtractorError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessorError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DownloaderError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ExtractorError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PostprocessorError(#[from] pub anyhow::Error);
