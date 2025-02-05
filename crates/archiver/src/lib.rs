use image::ImageError;
use s3::error::S3Error;
pub use thumbnail::{ThumbnailArchiver, ThumbnailData};

mod thumbnail;

#[async_trait::async_trait]
pub trait Archiver<Data>: Send + Sync + 'static {
    type Output;

    async fn archive(&self, data: Data) -> Result<Self::Output, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Storage(#[from] S3Error),

    #[error(transparent)]
    Image(#[from] ImageError),
}
