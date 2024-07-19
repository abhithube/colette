use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use http::Response;

#[async_trait::async_trait]
pub trait Downloader {
    async fn download(&self, url: &mut String) -> Result<Response<Bytes>, DownloadError>;
}

pub trait Extractor<T> {
    fn extract(&self, url: &str, raw: &str) -> Result<T, ExtractError>;
}

pub trait Postprocessor<T, U> {
    fn postprocess(&self, url: &str, extracted: T) -> Result<U, PostprocessError>;
}

#[async_trait::async_trait]
pub trait Scraper<T> {
    async fn scrape(&self, url: &mut String) -> Result<T, Error>;
}

pub struct PluginRegistry<T, U> {
    pub downloaders: HashMap<&'static str, Arc<dyn Downloader + Send + Sync>>,
    pub extractors: HashMap<&'static str, Arc<dyn Extractor<T> + Send + Sync>>,
    pub postprocessors: HashMap<&'static str, Arc<dyn Postprocessor<T, U> + Send + Sync>>,
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
