use colette_core::{bookmark::BookmarkScraper, feed::FeedScraper};
use colette_http::HyperClient;

#[allow(unused_variables)]
pub fn feeds(client: HyperClient) -> Vec<(&'static str, Box<dyn FeedScraper>)> {
    vec![]
}

#[allow(unused_variables)]
pub fn bookmarks(client: HyperClient) -> Vec<(&'static str, Box<dyn BookmarkScraper>)> {
    vec![]
}
