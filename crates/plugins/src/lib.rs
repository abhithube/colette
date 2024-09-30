use std::collections::HashMap;

use colette_scraper::{BookmarkPluginRegistry, BookmarkScraper, FeedPluginRegistry, FeedScraper};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins() -> FeedPluginRegistry {
    let plugins: HashMap<&str, Box<dyn FeedScraper>> = HashMap::from([
        ("www.reddit.com", reddit::feed()),
        ("www.youtube.com", youtube::create()),
    ]);

    FeedPluginRegistry::new(plugins)
}

pub fn register_bookmark_plugins() -> BookmarkPluginRegistry {
    let plugins: HashMap<&str, Box<dyn BookmarkScraper>> =
        HashMap::from([("www.reddit.com", reddit::bookmark())]);

    BookmarkPluginRegistry::new(plugins)
}
