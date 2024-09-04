use http::request::Parts;
use url::Url;

pub mod bookmark;
pub mod feed;
pub mod utils;

pub trait Scraper<T>: Send + Sync {
    fn scrape(&self, url: &mut Url) -> Result<T, Error>;
}

pub type DownloaderPlugin = fn(&mut Url) -> Result<Parts, DownloaderError>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Download(#[from] DownloaderError),

    #[error("failed to parse document")]
    Parse,

    #[error(transparent)]
    Extract(#[from] ExtractorError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessorError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DownloaderError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ExtractorError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PostprocessorError(#[from] pub anyhow::Error);
