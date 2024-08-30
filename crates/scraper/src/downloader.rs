use std::io::{BufRead, BufReader, Read};

use http::{request::Parts, HeaderMap, Request, Response};
use url::Url;

pub trait Downloader: Send + Sync {
    fn download(&self, url: &mut Url) -> Result<Response<Box<dyn BufRead>>, Error>;
}

pub type DownloaderFn = fn(&mut Url) -> Result<Parts, Error>;

pub enum DownloaderPlugin {
    Value(HeaderMap),
    Callback(DownloaderFn),
    Impl(Box<dyn Downloader>),
}

pub(crate) fn download(
    url: &mut Url,
    downloader: Option<&DownloaderPlugin>,
) -> Result<Response<Box<dyn BufRead>>, Error> {
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

            let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
            let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

            Ok(resp)
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] pub anyhow::Error);
