use std::collections::HashMap;

use colette_http::Client;
use colette_scraper::{
    bookmark::{BookmarkPluginRegistry, BookmarkScraper},
    downloader::Downloader,
    feed::{FeedPluginRegistry, FeedScraper},
};

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<D: Downloader + Clone, S: FeedScraper + Clone>(
    client: Client,
    downloader: D,
    default_scraper: S,
) -> FeedPluginRegistry<D, S> {
    let mut plugins = HashMap::from([("www.youtube.com", youtube::feed(default_scraper.clone()))]);
    plugins.extend(custom::feeds(
        client,
        downloader.clone(),
        default_scraper.clone(),
    ));

    FeedPluginRegistry::new(plugins, downloader, default_scraper)
}

pub fn register_bookmark_plugins<D: Downloader + Clone, S: BookmarkScraper + Clone>(
    client: Client,
    downloader: D,
    default_scraper: S,
) -> BookmarkPluginRegistry<S> {
    let mut plugins = HashMap::from([("www.reddit.com", reddit::bookmark(client.clone()))]);
    plugins.extend(custom::bookmarks(
        client,
        downloader,
        default_scraper.clone(),
    ));

    BookmarkPluginRegistry::new(plugins, default_scraper)
}
