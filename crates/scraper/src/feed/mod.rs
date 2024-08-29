use std::{collections::HashMap, sync::Arc};

pub use extractor::{
    DefaultFeedExtractor, ExtractedFeed, ExtractedFeedEntry, FeedExtractorOptions, HtmlExtractor,
};
use feed_detector::{DefaultFeedDetector, FeedDetector, FeedDetectorPlugin};
pub use postprocessor::{DefaultFeedPostprocessor, ProcessedFeed, ProcessedFeedEntry};
use url::Url;

use crate::{
    downloader::{download, DownloaderPlugin},
    extractor::{Extractor, ExtractorPlugin},
    postprocessor::{Postprocessor, PostprocessorPlugin},
    Scraper,
};

mod extractor;
pub mod feed_detector;
mod postprocessor;

pub trait FeedScraper: Scraper<ProcessedFeed> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error>;
}

#[derive(Default)]
pub struct FeedPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin>,
    pub detectors: HashMap<&'static str, FeedDetectorPlugin<'a>>,
    pub extractors: HashMap<&'static str, ExtractorPlugin<FeedExtractorOptions<'a>, ExtractedFeed>>,
    pub postprocessors:
        HashMap<&'static str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>>,
}

pub struct DefaultFeedScraper<'a> {
    registry: FeedPluginRegistry<'a>,
    default_detector: Arc<dyn FeedDetector>,
    default_extractor: Arc<dyn Extractor<Extracted = ExtractedFeed>>,
    default_postprocessor:
        Arc<dyn Postprocessor<Extracted = ExtractedFeed, Processed = ProcessedFeed>>,
}

impl<'a> DefaultFeedScraper<'a> {
    pub fn new(registry: FeedPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_detector: Arc::new(DefaultFeedDetector::new(None)),
            default_extractor: Arc::new(DefaultFeedExtractor {}),
            default_postprocessor: Arc::new(DefaultFeedPostprocessor {}),
        }
    }
}

impl Scraper<ProcessedFeed> for DefaultFeedScraper<'_> {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;

        let downloader = self.registry.downloaders.get(host);
        let extractor = self.registry.extractors.get(host);
        let postprocessor = self.registry.postprocessors.get(host);

        let resp = download(url, downloader)?;

        let extracted = match extractor {
            Some(ExtractorPlugin::Impl(extractor)) => extractor.extract(url, resp),
            _ => self.default_extractor.extract(url, resp),
        }?;

        let processed = match postprocessor {
            Some(PostprocessorPlugin::Impl(postprocessor)) => {
                postprocessor.postprocess(url, extracted)
            }
            _ => self.default_postprocessor.postprocess(url, extracted),
        }?;

        Ok(processed)
    }
}

impl FeedScraper for DefaultFeedScraper<'_> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;

        let downloader = self.registry.downloaders.get(host);
        let detector = self.registry.detectors.get(host);

        let resp = download(url, downloader)?;

        let detected = match detector {
            Some(FeedDetectorPlugin::Impl(detector)) => detector.detect(url, resp),
            _ => self.default_detector.detect(url, resp),
        }?;

        Ok(detected)
    }
}
