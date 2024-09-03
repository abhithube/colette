use std::io::{BufRead, BufReader, Read};

use http::{request::Parts, HeaderMap, Request, Response};
use url::Url;

use crate::DownloaderError;

pub trait Downloader: Send + Sync {
    fn download(&self, url: &mut Url) -> Result<Response<Box<dyn BufRead>>, DownloaderError>;
}

pub type DownloaderFn = fn(&mut Url) -> Result<Parts, DownloaderError>;

pub enum DownloaderPlugin {
    Value(HeaderMap),
    Callback(DownloaderFn),
    Impl(Box<dyn Downloader>),
}

pub struct DefaultDownloader;

impl Downloader for DefaultDownloader {
    fn download(&self, url: &mut Url) -> Result<Response<Box<dyn BufRead>>, DownloaderError> {
        let req = Request::get(url.as_str())
            .body(())
            .map_err(|e| DownloaderError(e.into()))?;

        let req: ureq::Request = req.into_parts().0.into();
        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
        let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

        Ok(resp)
    }
}
