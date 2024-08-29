use std::sync::Arc;

use http::{HeaderMap, Request, Response};
use url::Url;

pub trait Downloader: Send + Sync {
    fn download(&self, url: &mut Url) -> Result<Response<String>, Error> {
        let req: ureq::Request = Request::builder()
            .uri(url.as_str())
            .try_into()
            .map_err(|e: http::Error| Error(e.into()))?;

        let resp = req.call().map_err(|e| Error(e.into()))?;

        Ok(resp.into())
    }
}

pub type DownloaderFn<T> = fn(&str) -> Result<Request<T>, Error>;

pub enum DownloaderPlugin<T = ()> {
    Value(HeaderMap),
    Callback(DownloaderFn<T>),
    Impl(Arc<dyn Downloader>),
}

pub struct DefaultDownloader {}

impl Downloader for DefaultDownloader {}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
