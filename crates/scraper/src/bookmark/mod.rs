use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

pub use extractor::*;
use http::Response;
pub use postprocessor::*;
use scraper::Html;
use url::Url;

use crate::{
    utils::TextSelector, DownloaderError, DownloaderPlugin, ExtractorError, Scraper,
    DEFAULT_DOWNLOADER,
};

mod extractor;
mod postprocessor;

#[derive(Default)]
pub struct BookmarkPlugin<'a> {
    pub downloader: Option<DownloaderPlugin>,
    pub extractor: Option<BookmarkExtractorOptions<'a>>,
    pub postprocessor: Option<BookmarkPostprocessorPlugin>,
}

#[derive(Default)]
pub struct BookmarkPluginRegistry<'a> {
    pub scrapers: HashMap<&'static str, BookmarkPlugin<'a>>,
}

pub struct DefaultBookmarkScraper<'a> {
    registry: BookmarkPluginRegistry<'a>,
    default_downloader: DownloaderPlugin,
}

impl<'a> DefaultBookmarkScraper<'a> {
    pub fn new(registry: BookmarkPluginRegistry<'a>) -> Self {
        Self {
            registry,
            default_downloader: DEFAULT_DOWNLOADER,
        }
    }
}

impl Scraper<ProcessedBookmark> for DefaultBookmarkScraper<'_> {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;

        let plugin = self.registry.scrapers.get(host);

        let parts = (self.default_downloader)(url)?;
        let req: ureq::Request = parts.into();
        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
        let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

        let options = plugin.and_then(|e| e.extractor.clone()).unwrap_or_default();

        let mut body = resp.into_body();

        let mut bytes: Vec<u8> = vec![];
        body.read(&mut bytes)
            .map_err(|e| ExtractorError(e.into()))?;

        let raw = String::from_utf8_lossy(&bytes);
        let html = Html::parse_document(&raw);

        let extracted = ExtractedBookmark {
            title: html.select_text(&options.title_queries),
            thumbnail: html.select_text(&options.thumbnail_queries),
            published: html.select_text(&options.published_queries),
            author: html.select_text(&options.author_queries),
        };

        let processed = extracted.try_into()?;

        Ok(processed)
    }
}
