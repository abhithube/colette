use colette_scraper::{bookmark::BookmarkPlugin, feed::FeedPlugin};
use reqwest::Client;

#[allow(unused_variables)]
pub fn feeds(client: Client) -> Vec<(&'static str, Box<dyn FeedPlugin>)> {
    vec![]
}

#[allow(unused_variables)]
pub fn bookmarks(client: Client) -> Vec<(&'static str, Box<dyn BookmarkPlugin>)> {
    vec![]
}
