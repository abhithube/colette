use std::sync::Arc;

use http::{request::Parts, HeaderMap, Request, Response};
use url::Url;

pub trait Downloader: Send + Sync {
    fn download(&self, url: &mut Url) -> Result<Response<String>, Error>;

    fn download_from_parts(&self, parts: Parts) -> Result<Response<String>, Error> {
        let req: ureq::Request = parts.into();

        let resp = req.call().map_err(|e| Error(e.into()))?;

        Ok(resp.into())
    }
}

pub type DownloaderFn = fn(&mut Url) -> Result<Parts, Error>;

pub enum DownloaderPlugin {
    Value(HeaderMap),
    Callback(DownloaderFn),
    Impl(Arc<dyn Downloader>),
}

pub struct DefaultDownloader {}

impl Downloader for DefaultDownloader {
    fn download(&self, url: &mut Url) -> Result<Response<String>, Error> {
        let req = Request::get(url.as_str())
            .body(())
            .map_err(|e| Error(e.into()))?;

        self.download_from_parts(req.into_parts().0)
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
