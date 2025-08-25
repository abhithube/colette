use colette_scraper::{bookmark::BookmarkPlugin, feed::FeedPlugin};
use reddit::RedditBookmarkPlugin;
use reqwest::Client;

mod common;
mod custom;
mod reddit;

pub fn register_feed_plugins(client: Client) -> Vec<Box<dyn FeedPlugin>> {
    let mut plugins: Vec<Box<dyn FeedPlugin>> = vec![];

    plugins.extend(custom::feeds(client));

    plugins
}

pub fn register_bookmark_plugins(client: Client) -> Vec<Box<dyn BookmarkPlugin>> {
    let mut plugins: Vec<Box<dyn BookmarkPlugin>> =
        vec![Box::new(RedditBookmarkPlugin::new(client.clone()))];

    plugins.extend(custom::bookmarks(client));

    plugins
}
