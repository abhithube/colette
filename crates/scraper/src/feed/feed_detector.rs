use std::{io::BufRead, sync::Arc};

use http::Response;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    extractor,
    utils::{select, ExtractorQuery, Node},
};

pub trait FeedDetector: Send + Sync {
    fn detect(
        &self,
        url: &Url,
        resp: Response<Box<dyn BufRead>>,
    ) -> Result<Vec<Url>, extractor::Error>;
}

pub enum FeedDetectorPlugin<'a> {
    Value(Vec<ExtractorQuery<'a>>),
    Impl(Arc<dyn FeedDetector>),
}

pub struct DefaultFeedDetector<'a> {
    options: Vec<ExtractorQuery<'a>>,
}

impl<'a> DefaultFeedDetector<'a> {
    pub fn new(options: Option<Vec<ExtractorQuery<'a>>>) -> Self {
        Self {
            options: options.unwrap_or(vec![ExtractorQuery {
                selector: Selector::parse("link[type='application/rss+xml']").unwrap(),
                node: Node::Attr("href"),
            }]),
        }
    }
}

impl FeedDetector for DefaultFeedDetector<'_> {
    fn detect(
        &self,
        _url: &Url,
        resp: Response<Box<dyn BufRead>>,
    ) -> Result<Vec<Url>, extractor::Error> {
        let mut body = resp.into_body();

        let mut bytes: Vec<u8> = vec![];
        body.read(&mut bytes)
            .map_err(|e| extractor::Error(e.into()))?;

        let raw = String::from_utf8_lossy(&bytes);
        let html = Html::parse_document(&raw);

        let urls = self
            .options
            .iter()
            .filter_map(|opt| {
                html.select(&opt.selector)
                    .next()
                    .and_then(|e| select(e, &opt.node))
            })
            .map(|e| Url::parse(&e))
            .collect::<Result<_, _>>()
            .map_err(|e| extractor::Error(e.into()))?;

        Ok(urls)
    }
}
