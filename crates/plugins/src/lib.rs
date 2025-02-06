use std::collections::HashMap;

use colette_core::{bookmark::BookmarkScraper, feed::FeedScraper};
use colette_http::HyperClient;
use reddit::RedditBookmarkPlugin;
use youtube::YouTubeFeedPlugin;

mod common;
mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins(client: HyperClient) -> HashMap<&'static str, Box<dyn FeedScraper>> {
    let mut plugins: Vec<(&'static str, Box<dyn FeedScraper>)> = vec![(
        "www.youtube.com",
        Box::new(YouTubeFeedPlugin::new(client.clone())),
    )];

    plugins.extend(custom::feeds(client));

    plugins.into_iter().collect()
}

pub fn register_bookmark_plugins(
    client: HyperClient,
) -> HashMap<&'static str, Box<dyn BookmarkScraper>> {
    let mut plugins: Vec<(&'static str, Box<dyn BookmarkScraper>)> = vec![(
        "www.reddit.com",
        Box::new(RedditBookmarkPlugin::new(client.clone())),
    )];

    plugins.extend(custom::bookmarks(client));

    plugins.into_iter().collect()
}
