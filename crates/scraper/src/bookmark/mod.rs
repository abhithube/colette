use std::collections::HashMap;

use extractor::BookmarkExtractor;
pub use extractor::{
    BookmarkExtractorOptions, BookmarkExtractorPlugin, DefaultBookmarkExtractor, ExtractedBookmark,
};
pub use postprocessor::{BookmarkPostprocessorPlugin, ProcessedBookmark};
use url::Url;

use crate::{
    downloader::{DefaultDownloader, Downloader, DownloaderPlugin},
    Scraper,
};

mod extractor;
mod postprocessor;

#[derive(Default)]
pub struct BookmarkPluginRegistry<'a> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin>,
    pub extractors: HashMap<&'static str, BookmarkExtractorPlugin<'a>>,
    pub postprocessors: HashMap<&'static str, BookmarkPostprocessorPlugin>,
}

pub struct DefaultBookmarkScraper<'a> {
    registry: BookmarkPluginRegistry<'a>,
    default_downloader: Box<dyn Downloader>,
    default_extractor: Box<dyn BookmarkExtractor>,
}

impl<'a> DefaultBookmarkScraper<'a> {
    pub fn new(registry: BookmarkPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: Box::new(DefaultDownloader),
            default_extractor: Box::new(DefaultBookmarkExtractor::new(None)),
        }
    }
}

impl Scraper<ProcessedBookmark> for DefaultBookmarkScraper<'_> {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;

        let _downloader = self.registry.downloaders.get(host);
        let _extractor = self.registry.extractors.get(host);
        let _postprocessor = self.registry.postprocessors.get(host);

        let resp = self.default_downloader.download(url)?;
        let extracted = self.default_extractor.extract(url, resp)?;
        let processed = extracted.try_into()?;

        Ok(processed)
    }
}
