use async_trait::async_trait;
use bytes::Bytes;
use colette_core::scraper::downloader::{Downloader, Error};
use http::Response;

pub struct DefaultDownloader {}

#[async_trait]
impl Downloader for DefaultDownloader {
    async fn download(&self, url: String) -> Result<Response<Bytes>, Error> {
        let resp = reqwest::get(url).await.map_err(|e| Error(e.into()))?;
        let bytes = resp.bytes().await.map_err(|e| Error(e.into()))?;

        Ok(Response::new(bytes))
    }
}
