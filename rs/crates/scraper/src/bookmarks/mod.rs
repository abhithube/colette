use std::sync::Arc;

use colette_core::{
    bookmarks::{ExtractedBookmark, ProcessedBookmark},
    utils::scraper::{Downloader, Error, Extractor, PluginRegistry, Postprocessor, Scraper},
};
pub use extractor::DefaultBookmarkExtractor;
pub use options::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options,
};
pub use postprocessor::DefaultBookmarkPostprocessor;
use url::Url;

mod extractor;
mod options;
mod postprocessor;

pub struct BookmarkScraper {
    registry: PluginRegistry<ExtractedBookmark, ProcessedBookmark>,
    default_downloader: Arc<dyn Downloader>,
    default_extractor: Arc<dyn Extractor<ExtractedBookmark>>,
    default_postprocessor: Arc<dyn Postprocessor<ExtractedBookmark, ProcessedBookmark>>,
}

impl BookmarkScraper {
    pub fn new(
        registry: PluginRegistry<ExtractedBookmark, ProcessedBookmark>,
        default_downloader: Arc<dyn Downloader>,
        default_extractor: Arc<dyn Extractor<ExtractedBookmark>>,
        default_postprocessor: Arc<dyn Postprocessor<ExtractedBookmark, ProcessedBookmark>>,
    ) -> Self {
        Self {
            registry,
            default_downloader,
            default_extractor,
            default_postprocessor,
        }
    }
}

impl Scraper<ProcessedBookmark> for BookmarkScraper {
    fn scrape(&self, url: &mut String) -> Result<ProcessedBookmark, Error> {
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
        let (_, body) = resp.into_parts();

        let extracted = extractor.extract(url, &body)?;
        let processed = postprocessor.postprocess(url, extracted)?;

        Ok(processed)
    }
}
