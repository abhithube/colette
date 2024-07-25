use colette_core::{
    feeds::Detector,
    utils::scraper::{ExtractError, ExtractorQuery, Node},
};
use http::Response;
use scraper::{Html, Selector};

use super::extractor::select;

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
    fn detect(&self, _url: &str, resp: Response<String>) -> Result<Vec<String>, ExtractError> {
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
            .collect::<Vec<_>>();

        Ok(urls)
    }
}
