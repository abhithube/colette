use colette_scraper::{bookmark::BookmarkPlugin, feed::FeedPlugin};
use reqwest::Client;

#[allow(unused_variables)]
pub fn feeds(client: Client) -> Vec<Box<dyn FeedPlugin>> {
    vec![]
}

#[allow(unused_variables)]
pub fn bookmarks(client: Client) -> Vec<Box<dyn BookmarkPlugin>> {
    vec![]
}
