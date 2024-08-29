use std::sync::Arc;

use http::{request::Parts, HeaderMap, Request, Response};
use url::Url;

pub trait Downloader: Send + Sync {
    fn download(&self, url: &mut Url) -> Result<Response<String>, Error>;
}

pub type DownloaderFn = fn(&mut Url) -> Result<Parts, Error>;

pub enum DownloaderPlugin {
    Value(HeaderMap),
    Callback(DownloaderFn),
    Impl(Arc<dyn Downloader>),
}

pub(crate) fn download(
    url: &mut Url,
    downloader: Option<&DownloaderPlugin>,
) -> Result<Response<String>, Error> {
    match downloader {
        Some(DownloaderPlugin::Impl(downloader)) => downloader.download(url),
        _ => {
            let parts = match downloader {
                Some(DownloaderPlugin::Callback(func)) => func(url)?,
                Some(DownloaderPlugin::Value(headers)) => {
                    let mut builder = Request::get(url.as_str());
                    for (name, value) in headers {
                        builder = builder.header(name, value);
                    }
                    let req = builder.body(()).map_err(|e| Error(e.into()))?;

                    req.into_parts().0
                }
                _ => {
                    let req = Request::get(url.as_str())
                        .body(())
                        .map_err(|e| Error(e.into()))?;

                    req.into_parts().0
                }
            };

            let req: ureq::Request = parts.into();

            let resp = req.call().map_err(|e| Error(e.into()))?;

            Ok(resp.into())
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
