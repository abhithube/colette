use std::collections::HashMap;

pub use detector::*;
pub use extractor::*;
pub use postprocessor::*;
use scraper::Html;
use url::Url;

use crate::{
    downloader::{DefaultDownloader, Downloader, DownloaderPlugin},
    utils::TextSelector,
    ExtractorError, Scraper,
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
    default_downloader: Box<dyn Downloader>,
    default_extractor: Box<dyn FeedExtractor>,
    default_detector: Box<dyn FeedDetector>,
    default_postprocessor: Box<dyn FeedPostprocessor>,
}

impl<'a> DefaultFeedScraper<'a> {
    pub fn new(registry: FeedPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: Box::new(DefaultDownloader),
            default_extractor: Box::new(DefaultXmlFeedExtractor),
            default_detector: Box::new(DefaultFeedDetector::new(None)),
            default_postprocessor: Box::new(DefaultFeedPostprocessor),
        }
    }
}

impl Scraper<ProcessedFeed> for DefaultFeedScraper<'_> {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;
        let plugin = self.registry.scrapers.get(host);

        let resp = self.default_downloader.download(url)?;

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

        let processed = self.default_postprocessor.postprocess(url, extracted)?;

        Ok(processed)
    }
}

impl FeedScraper for DefaultFeedScraper<'_> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;
        let _plugin = self.registry.scrapers.get(host);

        let resp = self.default_downloader.download(url)?;
        let detected = self.default_detector.detect(url, resp)?;

        Ok(detected)
    }
}
