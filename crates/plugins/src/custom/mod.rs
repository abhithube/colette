use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};
use reqwest::Client;

#[allow(unused_variables)]
pub fn feeds<S: FeedScraper + Clone>(
    client: Client,
    default_scraper: S,
) -> Vec<(&'static str, Box<dyn FeedScraper>)> {
    vec![]
}

#[allow(unused_variables)]
pub fn bookmarks<S: BookmarkScraper + Clone>(
    client: Client,
    default_scraper: S,
) -> Vec<(&'static str, Box<dyn BookmarkScraper>)> {
    vec![]
}
