use std::collections::HashMap;

use colette_scraper::{
    bookmark::{BookmarkPluginRegistry, BookmarkScraper},
    downloader::Downloader,
    feed::{FeedPluginRegistry, FeedScraper},
};
#[allow(unused_imports)]
use custom::*;
use reqwest::Client;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins(
    downloader: Box<dyn Downloader>,
    default_scraper: Box<dyn FeedScraper>,
) -> FeedPluginRegistry {
    FeedPluginRegistry::new(
        HashMap::from([("www.youtube.com", youtube::create(default_scraper.clone()))]),
        downloader,
        default_scraper,
    )
}

pub fn register_bookmark_plugins(
    client: Client,
    default_scraper: Box<dyn BookmarkScraper>,
) -> BookmarkPluginRegistry {
    BookmarkPluginRegistry::new(
        HashMap::from([("www.reddit.com", reddit::bookmark(client.clone()))]),
        default_scraper,
    )
}
