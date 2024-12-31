use colette_http::Client;
use colette_scraper::{
    bookmark::{BookmarkPluginRegistry, BookmarkScraper},
    downloader::Downloader,
    feed::{FeedPluginRegistry, FeedScraper},
};
use reddit::RedditBookmarkPlugin;
use youtube::YouTubeFeedPlugin;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<D: Downloader + Clone, S: FeedScraper + Clone>(
    client: Client,
    downloader: D,
    default_scraper: S,
) -> FeedPluginRegistry<D, S> {
    let mut plugins: Vec<(&'static str, Box<dyn FeedScraper>)> = vec![(
        "www.youtube.com",
        Box::new(YouTubeFeedPlugin::new(default_scraper.clone())),
    )];

    plugins.extend(custom::feeds(
        client,
        downloader.clone(),
        default_scraper.clone(),
    ));

    FeedPluginRegistry::new(plugins.into_iter().collect(), downloader, default_scraper)
}

pub fn register_bookmark_plugins<D: Downloader + Clone, S: BookmarkScraper + Clone>(
    client: Client,
    downloader: D,
    default_scraper: S,
) -> BookmarkPluginRegistry<S> {
    let mut plugins: Vec<(&'static str, Box<dyn BookmarkScraper>)> = vec![(
        "www.reddit.com",
        Box::new(RedditBookmarkPlugin::new(client.clone())),
    )];

    plugins.extend(custom::bookmarks(
        client,
        downloader,
        default_scraper.clone(),
    ));

    BookmarkPluginRegistry::new(plugins.into_iter().collect(), default_scraper)
}
