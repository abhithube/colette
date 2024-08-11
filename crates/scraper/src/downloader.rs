use colette_core::scraper::{DownloadError, Downloader};
use http::{Request, Response};
use url::Url;

pub struct DefaultDownloader {}

impl Downloader for DefaultDownloader {
    fn download(&self, url: &mut Url) -> Result<Response<String>, DownloadError> {
        let req: ureq::Request = Request::builder()
            .uri(url.as_str())
            .try_into()
            .map_err(|e: http::Error| DownloadError(e.into()))?;

        let resp = req.call().map_err(|e| DownloadError(e.into()))?;

        Ok(resp.into())
    }
}
