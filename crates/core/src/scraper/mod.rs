pub mod downloader;
pub mod extractor;
pub mod postprocessor;

use async_trait::async_trait;
pub use downloader::Downloader;
pub use extractor::Extractor;
pub use postprocessor::Postprocessor;

#[async_trait]
pub trait Scraper<T> {
    async fn scrape(&self, url: &str) -> Result<T, Error>;
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
