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

pub fn register_feed_plugins<D: Downloader, S: FeedScraper + Clone>(
    downloader: D,
    default_scraper: S,
) -> FeedPluginRegistry<D, S> {
    FeedPluginRegistry::new(
        HashMap::from([("www.youtube.com", youtube::feed(default_scraper.clone()))]),
        downloader,
        default_scraper,
    )
}

pub fn register_bookmark_plugins<S: BookmarkScraper>(
    client: Client,
    default_scraper: S,
) -> BookmarkPluginRegistry<S> {
    BookmarkPluginRegistry::new(
        HashMap::from([("www.reddit.com", reddit::bookmark(client.clone()))]),
        default_scraper,
    )
}
