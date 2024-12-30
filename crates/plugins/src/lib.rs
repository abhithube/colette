use std::collections::HashMap;

use colette_http::Client;
use colette_scraper::{
    bookmark::{BookmarkPluginRegistry, BookmarkScraper},
    downloader::Downloader,
    feed::{FeedPluginRegistry, FeedScraper},
};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins(
    downloader: impl Downloader,
    default_scraper: impl FeedScraper + Clone,
) -> FeedPluginRegistry {
    FeedPluginRegistry::new(
        HashMap::from([(
            "www.youtube.com",
            youtube::create(Box::new(default_scraper.clone())),
        )]),
        downloader,
        default_scraper,
    )
}

pub fn register_bookmark_plugins(
    client: Client,
    default_scraper: impl BookmarkScraper,
) -> BookmarkPluginRegistry {
    BookmarkPluginRegistry::new(
        HashMap::from([("www.reddit.com", reddit::bookmark(client.clone()))]),
        default_scraper,
    )
}
