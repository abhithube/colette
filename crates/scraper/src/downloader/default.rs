use bytes::Bytes;
use reqwest::Client;
use url::Url;

use crate::{Downloader, DownloaderError};

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
        let resp = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        resp.bytes()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))
    }
}
