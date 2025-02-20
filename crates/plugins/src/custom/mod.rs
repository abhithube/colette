use colette_core::{bookmark::BookmarkScraper, feed::FeedScraper};
use reqwest::Client;

#[allow(unused_variables)]
pub fn feeds(client: Client) -> Vec<(&'static str, Box<dyn FeedScraper>)> {
    vec![]
}

#[allow(unused_variables)]
pub fn bookmarks(client: Client) -> Vec<(&'static str, Box<dyn BookmarkScraper>)> {
    vec![]
}
