use colette_core::{
    bookmark::{BookmarkPluginRegistry, BookmarkScraper},
    feed::{FeedPluginRegistry, FeedScraper},
};
use colette_http::HyperClient;
use reddit::RedditBookmarkPlugin;
use youtube::YouTubeFeedPlugin;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<S: FeedScraper + Clone>(
    client: HyperClient,
    default_scraper: S,
) -> FeedPluginRegistry<S> {
    let mut plugins: Vec<(&'static str, Box<dyn FeedScraper>)> = vec![(
        "www.youtube.com",
        Box::new(YouTubeFeedPlugin::new(default_scraper.clone())),
    )];

    plugins.extend(custom::feeds(client, default_scraper.clone()));

    FeedPluginRegistry::new(plugins.into_iter().collect(), default_scraper)
}

pub fn register_bookmark_plugins<S: BookmarkScraper + Clone>(
    client: HyperClient,
    default_scraper: S,
) -> BookmarkPluginRegistry<S> {
    let mut plugins: Vec<(&'static str, Box<dyn BookmarkScraper>)> = vec![(
        "www.reddit.com",
        Box::new(RedditBookmarkPlugin::new(client.clone())),
    )];

    plugins.extend(custom::bookmarks(client, default_scraper.clone()));

    BookmarkPluginRegistry::new(plugins.into_iter().collect(), default_scraper)
}
