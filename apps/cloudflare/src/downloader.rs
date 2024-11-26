use bytes::Bytes;
use colette_scraper::{downloader::Downloader, DownloaderError};
use reqwest::Client;
use url::Url;

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
        inner_download(url, &self.client).await
    }
}

#[worker::send]
async fn inner_download(url: &mut Url, client: &Client) -> Result<Bytes, DownloaderError> {
    let resp = client
        .get(url.as_str())
        .send()
        .await
        .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

    resp.bytes()
        .await
        .map_err(|e: reqwest::Error| DownloaderError(e.into()))
}
