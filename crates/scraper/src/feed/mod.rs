use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

pub use detector::*;
pub use extractor::*;
use http::Response;
pub use postprocessor::*;
use scraper::Html;
use url::Url;

use crate::{
    utils::TextSelector, DownloaderError, DownloaderPlugin, ExtractorError, Scraper,
    DEFAULT_DOWNLOADER,
};

mod detector;
mod extractor;
mod postprocessor;

pub trait FeedScraper: Scraper<ProcessedFeed> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error>;
}

#[derive(Default)]
pub struct FeedPlugin<'a> {
    pub downloader: Option<DownloaderPlugin>,
    pub detector: Option<FeedDetectorPlugin<'a>>,
    pub extractor: Option<FeedExtractorPlugin<'a>>,
    pub postprocessor: Option<FeedPostprocessorPlugin>,
}

#[derive(Default)]
pub struct FeedPluginRegistry<'a> {
    pub scrapers: HashMap<&'static str, FeedPlugin<'a>>,
}

pub struct DefaultFeedScraper<'a> {
    registry: FeedPluginRegistry<'a>,
    default_downloader: DownloaderPlugin,
    default_extractor: Box<dyn FeedExtractor>,
    default_detector: Box<dyn FeedDetector>,
}

impl<'a> DefaultFeedScraper<'a> {
    pub fn new(registry: FeedPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: DEFAULT_DOWNLOADER,
            default_extractor: Box::new(DefaultXmlFeedExtractor),
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
            match &plugin.extractor {
                Some(plugin) => match plugin {
                    FeedExtractorPlugin::Value(options) => {
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
                                published: element
                                    .select_text(&options.feed_entry_published_queries),
                                description: element
                                    .select_text(&options.feed_entry_description_queries),
                                author: element.select_text(&options.feed_entry_author_queries),
                                thumbnail: element
                                    .select_text(&options.feed_entry_thumbnail_queries),
                            })
                            .collect();

                        let feed = ExtractedFeed {
                            link: html.select_text(&options.feed_link_queries),
                            title: html.select_text(&options.feed_title_queries),
                            entries,
                        };

                        Ok(feed)
                    }
                    FeedExtractorPlugin::Callback(func) => func(url, resp),
                },
                None => self.default_extractor.extract(url, resp),
            }
        } else {
            self.default_extractor.extract(url, resp)
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
