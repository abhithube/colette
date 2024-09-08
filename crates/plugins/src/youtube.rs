use colette_scraper::FeedScraper;
use http::{request::Builder, Request};
use lazy_regex::regex_captures;
use url::Url;

pub struct YouTubePlugin;

pub fn create() -> Box<dyn FeedScraper> {
    Box::new(YouTubePlugin)
}

impl FeedScraper for YouTubePlugin {
    fn before_download(&self, url: &mut Url) -> Builder {
        if let Some((_, channel_id)) =
            regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.clone().as_str())
        {
            url.set_path("feeds/videos.xml");
            url.set_query(Some(&format!("channel_id={}", channel_id)));
        }

        Request::get(url.as_ref())
    }
}
