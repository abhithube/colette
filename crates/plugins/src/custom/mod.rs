use std::collections::HashMap;

use colette_http::Client;
use colette_scraper::{bookmark::BookmarkScraper, downloader::Downloader, feed::FeedScraper};

#[allow(unused_variables)]
pub fn feeds<D: Downloader + Clone, S: FeedScraper + Clone>(
    client: Client,
    downloader: D,
    default_scraper: S,
) -> HashMap<&'static str, Box<dyn FeedScraper>> {
    HashMap::from([])
}

#[allow(unused_variables)]
pub fn bookmarks<D: Downloader + Clone, S: BookmarkScraper + Clone>(
    client: Client,
    downloader: D,
    default_scraper: S,
) -> HashMap<&'static str, Box<dyn BookmarkScraper>> {
    HashMap::from([])
}
