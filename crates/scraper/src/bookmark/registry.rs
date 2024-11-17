use std::collections::HashMap;

use url::Url;

use crate::Error;

use super::{BookmarkScraper, ProcessedBookmark};

#[derive(Clone)]
pub struct BookmarkPluginRegistry {
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
    default_scraper: Box<dyn BookmarkScraper>,
}

impl BookmarkPluginRegistry {
    pub fn new(
        plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
        default_scraper: Box<dyn BookmarkScraper>,
    ) -> Self {
        Self {
            plugins,
            default_scraper,
        }
    }
}

#[async_trait::async_trait]
impl BookmarkScraper for BookmarkPluginRegistry {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, Error> {
        let host = url.host_str().ok_or(Error::Parse)?;

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url).await,
            None => self.default_scraper.scrape(url).await,
        }
    }
}
