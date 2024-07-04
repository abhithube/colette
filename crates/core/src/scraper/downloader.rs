use async_trait::async_trait;
use bytes::Bytes;
use http::Response;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);

#[async_trait]
pub trait Downloader {
    async fn download(&self, url: &str) -> Result<Response<Bytes>, Error>;
}
