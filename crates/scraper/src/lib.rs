pub use bookmark::*;
pub use feed::*;
use url::Url;

mod bookmark;
pub mod downloader;
pub mod extractor;
mod feed;
pub mod postprocessor;
pub mod utils;

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
    Extract(#[from] extractor::Error),

    #[error(transparent)]
    Postprocess(#[from] postprocessor::Error),
}
