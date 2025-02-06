use core::str;
use std::collections::HashMap;

use url::Url;

use super::{FeedScraper, ProcessedFeed, ScraperError};

pub struct FeedPluginRegistry<S> {
    plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
    default_scraper: S,
}

impl<S: FeedScraper> FeedPluginRegistry<S> {
    pub fn new(plugins: HashMap<&'static str, Box<dyn FeedScraper>>, default_scraper: S) -> Self {
        Self {
            plugins,
            default_scraper,
        }
    }
}

#[async_trait::async_trait]
impl<S: FeedScraper + Clone> FeedScraper for FeedPluginRegistry<S> {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, ScraperError> {
        let host = url.host_str().unwrap();

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url).await,
            None => self.default_scraper.scrape(url).await,
        }
    }
}
