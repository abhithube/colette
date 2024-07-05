use async_trait::async_trait;
pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, ExtractorOptions};
use colette_core::{
    feeds::ProcessedFeed,
    scraper::{Downloader, Error, Extractor, Postprocessor, Scraper},
};
pub use downloader::DefaultDownloader;
pub use extractor::DefaultFeedExtractor;
pub use options::{AtomExtractorOptions, RssExtractorOptions};
pub use postprocessor::DefaultFeedPostprocessor;
pub use registry::PluginRegistry;
use url::Url;

mod downloader;
mod extractor;
mod options;
mod postprocessor;
mod registry;

pub struct FeedScraper {
    pub registry: PluginRegistry<ExtractedFeed, ProcessedFeed>,
    pub default_downloader: Box<dyn Downloader + Send + Sync>,
    pub default_extractor: Box<dyn Extractor<ExtractedFeed> + Send + Sync>,
    pub default_postprocessor: Box<dyn Postprocessor<ExtractedFeed, ProcessedFeed> + Send + Sync>,
}

#[async_trait]
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
