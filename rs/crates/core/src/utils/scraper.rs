use std::{collections::HashMap, sync::Arc};

use http::Response;

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

pub trait Extractor<T>: Send + Sync {
    fn extract(&self, url: &str, raw: &str) -> Result<T, ExtractError>;
}

pub trait Postprocessor<T, U>: Send + Sync {
    fn postprocess(&self, url: &str, extracted: T) -> Result<U, PostprocessError>;
}

pub trait Scraper<T>: Send + Sync {
    fn scrape(&self, url: &mut String) -> Result<T, Error>;
}

pub struct PluginRegistry<T, U> {
    pub downloaders: HashMap<&'static str, Arc<dyn Downloader>>,
    pub extractors: HashMap<&'static str, Arc<dyn Extractor<T>>>,
    pub postprocessors: HashMap<&'static str, Arc<dyn Postprocessor<T, U>>>,
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
