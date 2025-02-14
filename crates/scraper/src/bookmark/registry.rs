use std::collections::HashMap;

use url::Url;

use super::{BookmarkScraper, ProcessedBookmark};
use crate::Error;

pub struct BookmarkPluginRegistry<S> {
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
    default_scraper: S,
}

impl<S: BookmarkScraper> BookmarkPluginRegistry<S> {
    pub fn new(
        plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
        default_scraper: S,
    ) -> Self {
        Self {
            plugins,
            default_scraper,
        }
    }
}

#[async_trait::async_trait]
impl<S: BookmarkScraper + Clone> BookmarkScraper for BookmarkPluginRegistry<S> {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, Error> {
        let host = url.host_str().unwrap();

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url).await,
            None => self.default_scraper.scrape(url).await,
        }
    }
}
