use std::collections::HashMap;

use colette_scraper::{bookmark::BookmarkPluginRegistry, FeedPluginRegistry, FeedScraper};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins() -> FeedPluginRegistry {
    let plugins: HashMap<&str, Box<dyn FeedScraper>> = HashMap::from([
        ("www.reddit.com", reddit::create()),
        ("www.youtube.com", youtube::create()),
    ]);

    FeedPluginRegistry::new(plugins)
}

pub fn register_bookmark_plugins() -> BookmarkPluginRegistry {
    BookmarkPluginRegistry::default()
}
