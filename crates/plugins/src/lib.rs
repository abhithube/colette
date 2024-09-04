use std::collections::HashMap;

use colette_scraper::{bookmark::BookmarkPlugin, feed::FeedPlugin};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<'a>() -> HashMap<&'static str, FeedPlugin<'a>> {
    HashMap::from([
        ("www.youtube.com", youtube::new_youtube_feed_plugin()),
        ("www.reddit.com", reddit::new_reddit_feed_plugin()),
    ])
}

pub fn register_bookmark_plugins<'a>() -> HashMap<&'static str, BookmarkPlugin<'a>> {
    HashMap::new()
}
