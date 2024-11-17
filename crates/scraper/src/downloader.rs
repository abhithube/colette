use bytes::Bytes;
use dyn_clone::DynClone;
use reqwest::Client;
use url::Url;

use crate::DownloaderError;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync + DynClone {
    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError>;
}

dyn_clone::clone_trait_object!(Downloader);

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
