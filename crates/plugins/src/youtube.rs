use colette_core::feed::{FeedScraper, ProcessedFeed, ScraperError};
use lazy_regex::regex_captures;
use url::Url;

#[derive(Clone)]
pub struct YouTubeFeedPlugin<S> {
    default_scraper: S,
}

impl<S: FeedScraper> YouTubeFeedPlugin<S> {
    pub fn new(default_scraper: S) -> Self {
        Self { default_scraper }
    }
}

#[async_trait::async_trait]
impl<S: FeedScraper + Clone> FeedScraper for YouTubeFeedPlugin<S> {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, ScraperError> {
        if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.as_str()) {
            url.set_query(Some(&format!("channel_id={}", channel_id)));
            url.set_path("feeds/videos.xml");
        }

        self.default_scraper.scrape(url).await
    }
}
