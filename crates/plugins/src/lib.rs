use std::collections::HashMap;

use colette_scraper::{BookmarkPluginRegistry, Downloader, FeedPluginRegistry};
#[allow(unused_imports)]
use custom::*;
use reqwest::Client;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins(
    client: Client,
    downloader: Box<dyn Downloader>,
) -> FeedPluginRegistry {
    FeedPluginRegistry::new(
        HashMap::from([("www.youtube.com", youtube::create(client.clone()))]),
        downloader,
    )
}

pub fn register_bookmark_plugins(
    client: Client,
    downloader: Box<dyn Downloader>,
) -> BookmarkPluginRegistry {
    BookmarkPluginRegistry::new(
        HashMap::from([("www.reddit.com", reddit::bookmark(client.clone()))]),
        downloader,
    )
}
