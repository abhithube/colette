use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

pub use extractor::*;
use http::Response;
pub use postprocessor::*;
use url::Url;

use crate::{DownloaderError, DownloaderPlugin, Scraper, DEFAULT_DOWNLOADER};

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
    default_downloader: DownloaderPlugin,
    default_extractor: Box<dyn BookmarkExtractor>,
}

impl<'a> DefaultBookmarkScraper<'a> {
    pub fn new(registry: BookmarkPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: DEFAULT_DOWNLOADER,
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

        let parts = (self.default_downloader)(url)?;
        let req: ureq::Request = parts.into();
        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
        let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

        let extracted = self.default_extractor.extract(url, resp)?;
        let processed = extracted.try_into()?;

        Ok(processed)
    }
}
