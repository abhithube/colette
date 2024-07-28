use colette_core::utils::scraper::{DownloadError, Downloader};
use http::{Request, Response};

pub struct DefaultDownloader {}

impl Downloader for DefaultDownloader {
    fn download(&self, url: &mut String) -> Result<Response<String>, DownloadError> {
        let req: ureq::Request = Request::builder()
            .uri(url.as_str())
            .try_into()
            .map_err(|e: http::Error| DownloadError(e.into()))?;

        let resp = req.call().map_err(|e| DownloadError(e.into()))?;

        Ok(resp.into())
    }
}