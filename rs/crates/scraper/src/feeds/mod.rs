use std::sync::Arc;

use colette_core::{
    feeds::{ExtractedFeed, ProcessedFeed},
    utils::scraper::{Downloader, Error, Extractor, PluginRegistry, Postprocessor, Scraper},
};
use extractor::DefaultFeedExtractor;
pub use extractor::{HtmlExtractor, TextSelector};
pub use postprocessor::DefaultFeedPostprocessor;
use url::Url;

use crate::DefaultDownloader;

mod atom;
mod extractor;
mod postprocessor;
mod rss;

pub struct FeedScraper {
    registry: PluginRegistry<ExtractedFeed, ProcessedFeed>,
    default_downloader: Arc<dyn Downloader>,
    default_extractor: Arc<dyn Extractor<ExtractedFeed>>,
    default_postprocessor: Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed>>,
}

impl FeedScraper {
    pub fn new(registry: PluginRegistry<ExtractedFeed, ProcessedFeed>) -> Self {
        Self {
            registry,
            default_downloader: Arc::new(DefaultDownloader {}),
            default_extractor: Arc::new(DefaultFeedExtractor {}),
            default_postprocessor: Arc::new(DefaultFeedPostprocessor {}),
        }
    }
}

impl Scraper<ProcessedFeed> for FeedScraper {
    fn scrape(&self, url: &mut String) -> Result<ProcessedFeed, Error> {
        let parsed = Url::parse(url).map_err(|_| Error::Parse)?;
        let host = parsed.host_str().ok_or(Error::Parse)?;

        let downloader = self
            .registry
            .downloaders
            .get(host)
            .unwrap_or(&self.default_downloader);
        let extractor = self
            .registry
            .extractors
            .get(host)
            .unwrap_or(&self.default_extractor);
        let postprocessor = self
            .registry
            .postprocessors
            .get(host)
            .unwrap_or(&self.default_postprocessor);

        let resp = downloader.download(url)?;
        let extracted = extractor.extract(url, resp)?;
        let processed = postprocessor.postprocess(url, extracted)?;

        Ok(processed)
    }
}
