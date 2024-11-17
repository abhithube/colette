use colette_scraper::{FeedScraper, ProcessedFeed};
use lazy_regex::regex_captures;
use url::Url;

#[derive(Clone)]
pub struct YouTubeFeedPlugin {
    default_scraper: Box<dyn FeedScraper>,
}

pub fn create(default_scraper: Box<dyn FeedScraper>) -> Box<dyn FeedScraper> {
    Box::new(YouTubeFeedPlugin { default_scraper })
}

#[async_trait::async_trait]
impl FeedScraper for YouTubeFeedPlugin {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, colette_scraper::Error> {
        if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.as_str()) {
            url.set_query(Some(&format!("channel_id={}", channel_id)));
            url.set_path("feeds/videos.xml");
        }

        self.default_scraper.scrape(url).await
    }
}
