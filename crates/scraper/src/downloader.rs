use crate::DownloaderError;
use bytes::Bytes;
use colette_http::Client;
use url::Url;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync + 'static {
    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError>;
}

#[derive(Clone)]
pub struct DefaultDownloader {
    client: Client,
}

impl DefaultDownloader {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl Downloader for DefaultDownloader {
    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError> {
        let body = self
            .client
            .get(url.as_str(), None)
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        Ok(body)
    }
}
