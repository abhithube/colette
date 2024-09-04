use std::collections::HashMap;

use colette_scraper::{bookmark::BookmarkPluginRegistry, feed::FeedPluginRegistry};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<'a>() -> FeedPluginRegistry<'a> {
    let scrapers = HashMap::from([
        ("www.youtube.com", youtube::new_youtube_feed_plugin()),
        ("www.reddit.com", reddit::new_reddit_feed_plugin()),
    ]);

    FeedPluginRegistry { scrapers }
}

pub fn register_bookmark_plugins<'a>() -> BookmarkPluginRegistry<'a> {
    let scrapers = HashMap::new();

    BookmarkPluginRegistry { scrapers }
}
