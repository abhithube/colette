use std::collections::HashMap;

use colette_scraper::{bookmark::BookmarkPlugin, feed::FeedPlugin};
use reddit::RedditBookmarkPlugin;
use reqwest::Client;
use youtube::YouTubeFeedPlugin;

mod common;
mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins(client: Client) -> HashMap<&'static str, Box<dyn FeedPlugin>> {
    let mut plugins: Vec<(&'static str, Box<dyn FeedPlugin>)> = vec![(
        "www.youtube.com",
        Box::new(YouTubeFeedPlugin::new(client.clone())),
    )];

    plugins.extend(custom::feeds(client));

    plugins.into_iter().collect()
}

pub fn register_bookmark_plugins(client: Client) -> HashMap<&'static str, Box<dyn BookmarkPlugin>> {
    let mut plugins: Vec<(&'static str, Box<dyn BookmarkPlugin>)> = vec![(
        "www.reddit.com",
        Box::new(RedditBookmarkPlugin::new(client.clone())),
    )];

    plugins.extend(custom::bookmarks(client));

    plugins.into_iter().collect()
}
