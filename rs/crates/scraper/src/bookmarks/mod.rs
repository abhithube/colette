use std::sync::Arc;

use colette_core::{
    bookmarks::{ExtractedBookmark, ProcessedBookmark},
    utils::scraper::{Downloader, Error, Extractor, Postprocessor, Scraper},
};
pub use extractor::DefaultBookmarkExtractor;
pub use options::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options,
};
pub use postprocessor::DefaultBookmarkPostprocessor;
use url::Url;

use crate::PluginRegistry;

mod extractor;
mod options;
mod postprocessor;

pub struct BookmarkScraper {
    registry: PluginRegistry<ExtractedBookmark, ProcessedBookmark>,
    default_downloader: Arc<dyn Downloader + Send + Sync>,
    default_extractor: Arc<dyn Extractor<ExtractedBookmark> + Send + Sync>,
    default_postprocessor:
        Arc<dyn Postprocessor<ExtractedBookmark, ProcessedBookmark> + Send + Sync>,
}

impl BookmarkScraper {
    pub fn new(
        registry: PluginRegistry<ExtractedBookmark, ProcessedBookmark>,
        default_downloader: Arc<dyn Downloader + Send + Sync>,
        default_extractor: Arc<dyn Extractor<ExtractedBookmark> + Send + Sync>,
        default_postprocessor: Arc<
            dyn Postprocessor<ExtractedBookmark, ProcessedBookmark> + Send + Sync,
        >,
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
impl Scraper<ProcessedBookmark> for BookmarkScraper {
    async fn scrape(&self, url: &mut String) -> Result<ProcessedBookmark, Error> {
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
