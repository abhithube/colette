use std::sync::Arc;

use http::Response;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    extractor,
    utils::{select, ExtractorQuery, Node},
};

pub trait Detector: Send + Sync {
    fn detect(&self, url: &Url, resp: Response<String>) -> Result<Vec<Url>, extractor::Error>;
}

pub enum DetectorPlugin<'a> {
    Value(Vec<ExtractorQuery<'a>>),
    Impl(Arc<dyn Detector>),
}

pub struct DefaultFeedDetector<'a> {
    options: Vec<ExtractorQuery<'a>>,
}

impl<'a> DefaultFeedDetector<'a> {
    pub fn new(options: Option<Vec<ExtractorQuery<'a>>>) -> Self {
        Self {
            options: options.unwrap_or(vec![ExtractorQuery {
                selector: "link[type='application/rss+xml']",
                node: Node::Attr("href"),
            }]),
        }
    }
}

impl Detector for DefaultFeedDetector<'_> {
    fn detect(&self, _url: &Url, resp: Response<String>) -> Result<Vec<Url>, extractor::Error> {
        let raw = resp.into_body();
        let html = Html::parse_document(&raw);

        let urls = self
            .options
            .iter()
            .filter_map(|opt| {
                html.select(&Selector::parse(opt.selector).unwrap())
                    .next()
                    .and_then(|e| select(e, &opt.node))
            })
            .map(|e| Url::parse(&e))
            .collect::<Result<_, _>>()
            .map_err(|e| extractor::Error(e.into()))?;

        Ok(urls)
    }
}
