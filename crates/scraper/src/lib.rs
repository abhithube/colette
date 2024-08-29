use std::sync::Arc;

pub use bookmark::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options, BookmarkExtractorOptions, BookmarkPluginRegistry,
    DefaultBookmarkExtractor, DefaultBookmarkPostprocessor, DefaultBookmarkScraper,
    ExtractedBookmark, ProcessedBookmark,
};
pub use feed::{
    DefaultFeedPostprocessor, DefaultFeedScraper, DetectorPlugin, ExtractedFeed,
    FeedExtractorOptions, FeedPluginRegistry, FeedScraper, HtmlExtractor, ProcessedFeed,
};
use http::Response;
use url::Url;

mod bookmark;
pub mod downloader;
mod feed;
mod utils;

#[derive(Clone, Debug)]
pub enum Node<'a> {
    Text,
    Attr(&'a str),
}

#[derive(Clone, Debug)]
pub struct ExtractorQuery<'a> {
    pub selector: &'a str,
    pub node: Node<'a>,
}

impl<'a> ExtractorQuery<'a> {
    pub fn new(selector: &'a str, node: Node<'a>) -> Self {
        Self { selector, node }
    }
}

pub trait Extractor: Send + Sync {
    type T;

    fn extract(&self, url: &Url, resp: Response<String>) -> Result<Self::T, ExtractError>;
}

pub enum ExtractorPlugin<T, U> {
    Value(T),
    Impl(Arc<dyn Extractor<T = U>>),
}

pub trait Postprocessor: Send + Sync {
    type T;
    type U;

    fn postprocess(&self, url: &Url, extracted: Self::T) -> Result<Self::U, PostprocessError>;
}

pub enum PostprocessorPlugin<T, U, V> {
    Value(U),
    Impl(Arc<dyn Postprocessor<T = T, U = V>>),
}

pub trait Scraper<T>: Send + Sync {
    fn scrape(&self, url: &mut Url) -> Result<T, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Download(#[from] downloader::Error),

    #[error("failed to parse document")]
    Parse,

    #[error(transparent)]
    Extract(#[from] ExtractError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ExtractError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PostprocessError(#[from] pub anyhow::Error);
