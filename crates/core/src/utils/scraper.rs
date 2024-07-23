use std::{collections::HashMap, sync::Arc};

use http::{HeaderMap, Request, Response};

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

pub trait Downloader: Send + Sync {
    fn download(&self, url: &mut String) -> Result<Response<String>, DownloadError>;
}

pub type DownloaderFn<T> = fn(&str) -> Result<Request<T>, DownloadError>;

pub enum DownloaderPlugin<T = ()> {
    Value(HeaderMap),
    Callback(DownloaderFn<T>),
    Impl(Arc<dyn Downloader>),
}

pub trait Extractor: Send + Sync {
    type T;

    fn extract(&self, url: &str, resp: Response<String>) -> Result<Self::T, ExtractError>;
}

pub enum ExtractorPlugin<T, U> {
    Value(T),
    Impl(Arc<dyn Extractor<T = U>>),
}

pub trait Postprocessor: Send + Sync {
    type T;
    type U;

    fn postprocess(&self, url: &str, extracted: Self::T) -> Result<Self::U, PostprocessError>;
}

pub enum PostprocessorPlugin<T, U, V> {
    Value(U),
    Impl(Arc<dyn Postprocessor<T = T, U = V>>),
}

pub trait Scraper<T>: Send + Sync {
    fn scrape(&self, url: &mut String) -> Result<T, Error>;
}

pub struct PluginRegistry<T, U, V, W, X = ()> {
    pub downloaders: HashMap<&'static str, DownloaderPlugin<X>>,
    pub extractors: HashMap<&'static str, ExtractorPlugin<T, U>>,
    pub postprocessors: HashMap<&'static str, PostprocessorPlugin<U, V, W>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Download(#[from] DownloadError),

    #[error("failed to parse document")]
    Parse,

    #[error(transparent)]
    Extract(#[from] ExtractError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DownloadError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ExtractError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PostprocessError(#[from] pub anyhow::Error);
