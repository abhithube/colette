use std::io::Read;

use http::{request::Builder, Request, Response};
use url::Url;

use crate::DownloaderError;

pub trait Downloader: Send + Sync {
    fn before_download(&self, url: &mut Url) -> Builder {
        Request::get(url.as_str())
    }

    fn download(
        &self,
        url: &mut Url,
    ) -> Result<Response<Box<dyn Read + Send + Sync>>, DownloaderError> {
        let req: ureq::Request = self
            .before_download(url)
            .try_into()
            .map_err(|e: http::Error| DownloaderError(e.into()))?;

        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        Ok(resp.into())
    }
}
