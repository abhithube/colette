pub use bookmark::*;
use url::Url;

mod bookmark;
pub mod downloader;
pub mod feed;
pub mod utils;

pub trait Scraper<T>: Send + Sync {
    fn scrape(&self, url: &mut Url) -> Result<T, Error>;
}

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
