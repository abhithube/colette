use bytes::Bytes;
use http::{request::Builder, Request};
use url::Url;

use crate::DownloaderError;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    fn before_download(&self, url: &mut Url) -> Builder {
        Request::get(url.as_str())
    }

    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError> {
        let req = reqwest::Request::try_from(self.before_download(url).body("").unwrap())
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        let resp = reqwest::Client::new()
            .execute(req)
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        resp.bytes()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))
    }
}
