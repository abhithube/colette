use std::{collections::HashMap, sync::Arc};

use detector::{DefaultFeedDetector, Detector, DetectorPlugin};
pub use extractor::{
    DefaultFeedExtractor, ExtractedFeed, ExtractedFeedEntry, FeedExtractorOptions, HtmlExtractor,
};
pub use postprocessor::{DefaultFeedPostprocessor, ProcessedFeed, ProcessedFeedEntry};
use url::Url;

use crate::{
    downloader::{DefaultDownloader, Downloader, DownloaderPlugin},
    extractor::{Extractor, ExtractorPlugin},
    postprocessor::{Postprocessor, PostprocessorPlugin},
    Scraper,
};

mod atom;
pub mod detector;
mod extractor;
mod postprocessor;
mod rss;

pub trait FeedScraper: Scraper<ProcessedFeed> {
    fn detect(&self, url: &mut Url) -> Result<Vec<Url>, crate::Error>;
}

#[derive(Default)]
pub struct FeedPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin<()>>,
    pub detectors: HashMap<&'static str, DetectorPlugin<'a>>,
    pub extractors: HashMap<&'static str, ExtractorPlugin<FeedExtractorOptions<'a>, ExtractedFeed>>,
    pub postprocessors:
        HashMap<&'static str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>>,
}

pub struct DefaultFeedScraper<'a> {
    registry: FeedPluginRegistry<'a>,
    default_downloader: Arc<dyn Downloader>,
    default_detector: Arc<dyn Detector>,
    default_extractor: Arc<dyn Extractor<T = ExtractedFeed>>,
    default_postprocessor: Arc<dyn Postprocessor<T = ExtractedFeed, U = ProcessedFeed>>,
}

impl<'a> DefaultFeedScraper<'a> {
    pub fn new(registry: FeedPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_detector: Arc::new(DefaultFeedDetector::new(None)),
            default_downloader: Arc::new(DefaultDownloader {}),
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

        let resp = match downloader {
            Some(DownloaderPlugin::Impl(downloader)) => downloader.download(url),
            _ => self.default_downloader.download(url),
        }?;

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

        let resp = match downloader {
            Some(DownloaderPlugin::Impl(downloader)) => downloader.download(url),
            _ => self.default_downloader.download(url),
        }?;

        let detected = match detector {
            Some(DetectorPlugin::Impl(detector)) => detector.detect(url, resp),
            _ => self.default_detector.detect(url, resp),
        }?;

        Ok(detected)
    }
}
