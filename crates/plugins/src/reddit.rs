use colette_scraper::FeedScraper;
use http::{header, request::Builder, Request};
use url::Url;

pub struct RedditPlugin;

pub fn create() -> Box<dyn FeedScraper> {
    Box::new(RedditPlugin)
}

impl FeedScraper for RedditPlugin {
    fn before_download(&self, url: &mut Url) -> Builder {
        if !url.path().contains(".rss") {
            url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
        }

        Request::get(url.as_ref())
            .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
    }
}
