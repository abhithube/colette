use colette_http::HyperClient;
use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};

#[allow(unused_variables)]
pub fn feeds<S: FeedScraper + Clone>(
    client: HyperClient,
    default_scraper: S,
) -> Vec<(&'static str, Box<dyn FeedScraper>)> {
    vec![]
}

#[allow(unused_variables)]
pub fn bookmarks<S: BookmarkScraper + Clone>(
    client: HyperClient,
    default_scraper: S,
) -> Vec<(&'static str, Box<dyn BookmarkScraper>)> {
    vec![]
}
