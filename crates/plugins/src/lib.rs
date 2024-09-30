use std::collections::HashMap;

use colette_scraper::{BookmarkPluginRegistry, FeedPluginRegistry};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins() -> FeedPluginRegistry {
    FeedPluginRegistry::new(HashMap::from([
        ("www.reddit.com", reddit::feed()),
        ("www.youtube.com", youtube::create()),
    ]))
}

pub fn register_bookmark_plugins() -> BookmarkPluginRegistry {
    BookmarkPluginRegistry::new(HashMap::from([("www.reddit.com", reddit::bookmark())]))
}
