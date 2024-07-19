use bytes::Bytes;
use colette_core::utils::scraper::{DownloadError, Downloader};
use http::Response;

pub struct DefaultDownloader {}

#[async_trait::async_trait]
impl Downloader for DefaultDownloader {
    async fn download(&self, url: &mut String) -> Result<Response<Bytes>, DownloadError> {
        let resp = reqwest::get(url.as_str())
            .await
            .map_err(|e| DownloadError(e.into()))?;
        let bytes = resp.bytes().await.map_err(|e| DownloadError(e.into()))?;

        Ok(Response::new(bytes))
    }
}
