use std::sync::Arc;

use colette_core::{
    feeds::{ExtractedFeed, ProcessedFeed},
    utils::scraper::{Downloader, Error, Extractor, Postprocessor, Scraper},
};
pub use extractor::DefaultFeedExtractor;
pub use options::{
    atom_extractor_options, dublin_core_extractor_options, itunes_extractor_options,
    media_extractor_options, rss_extractor_options,
};
pub use postprocessor::DefaultFeedPostprocessor;
use url::Url;

use crate::PluginRegistry;

mod extractor;
mod options;
mod postprocessor;

pub struct FeedScraper {
    registry: PluginRegistry<ExtractedFeed, ProcessedFeed>,
    default_downloader: Arc<dyn Downloader + Send + Sync>,
    default_extractor: Arc<dyn Extractor<ExtractedFeed> + Send + Sync>,
    default_postprocessor: Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed> + Send + Sync>,
}

impl FeedScraper {
    pub fn new(
        registry: PluginRegistry<ExtractedFeed, ProcessedFeed>,
        default_downloader: Arc<dyn Downloader + Send + Sync>,
        default_extractor: Arc<dyn Extractor<ExtractedFeed> + Send + Sync>,
        default_postprocessor: Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed> + Send + Sync>,
    ) -> Self {
        Self {
            registry,
            default_downloader,
            default_extractor,
            default_postprocessor,
        }
    }
}

#[async_trait::async_trait]
impl Scraper<ProcessedFeed> for FeedScraper {
    async fn scrape(&self, url: &str) -> Result<ProcessedFeed, Error> {
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

        let resp = downloader.download(url).await?;
        let body = String::from_utf8_lossy(resp.body());

        let extracted = extractor.extract(url, &body)?;
        let processed = postprocessor.postprocess(url, extracted)?;

        Ok(processed)
    }
}
