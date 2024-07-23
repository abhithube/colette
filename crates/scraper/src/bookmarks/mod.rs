use std::sync::Arc;

use colette_core::{
    bookmarks::{BookmarkExtractorOptions, ExtractedBookmark, ProcessedBookmark},
    utils::scraper::{
        Downloader, DownloaderPlugin, Error, Extractor, ExtractorPlugin, PluginRegistry,
        Postprocessor, PostprocessorPlugin, Scraper,
    },
};
pub use extractor::DefaultBookmarkExtractor;
pub use options::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options,
};
pub use postprocessor::DefaultBookmarkPostprocessor;
use url::Url;

use crate::DefaultDownloader;

mod extractor;
mod options;
mod postprocessor;

pub struct BookmarkScraper<'a> {
    registry:
        PluginRegistry<BookmarkExtractorOptions<'a>, ExtractedBookmark, (), ProcessedBookmark>,
    default_downloader: Arc<dyn Downloader>,
    default_extractor: Arc<dyn Extractor<T = ExtractedBookmark>>,
    default_postprocessor: Arc<dyn Postprocessor<T = ExtractedBookmark, U = ProcessedBookmark>>,
}

impl<'a> BookmarkScraper<'a> {
    pub fn new(
        registry: PluginRegistry<
            BookmarkExtractorOptions<'a>,
            ExtractedBookmark,
            (),
            ProcessedBookmark,
        >,
    ) -> Self {
        Self {
            registry,
            default_downloader: Arc::new(DefaultDownloader {}),
            default_extractor: Arc::new(DefaultBookmarkExtractor::new(None)),
            default_postprocessor: Arc::new(DefaultBookmarkPostprocessor {}),
        }
    }
}

impl Scraper<ProcessedBookmark> for BookmarkScraper<'_> {
    fn scrape(&self, url: &mut String) -> Result<ProcessedBookmark, Error> {
        let parsed = Url::parse(url).map_err(|_| Error::Parse)?;
        let host = parsed.host_str().ok_or(Error::Parse)?;

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
