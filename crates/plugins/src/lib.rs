use std::collections::HashMap;

use colette_scraper::{
    downloader::DownloaderPlugin, feed::FeedPluginRegistry, BookmarkExtractorPlugin,
    BookmarkPluginRegistry, BookmarkPostprocessorPlugin,
};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<'a>() -> FeedPluginRegistry<'a> {
    let scrapers = HashMap::from([
        ("www.youtube.com", youtube::new_youtube_feed_plugin()),
        ("www.reddit.com", reddit::new_reddit_feed_plugin()),
    ]);

    FeedPluginRegistry { scrapers }
}

pub fn register_bookmark_plugins<'a>() -> BookmarkPluginRegistry<'a> {
    let downloaders: HashMap<&str, DownloaderPlugin> = HashMap::new();
    let extractors: HashMap<&str, BookmarkExtractorPlugin> = HashMap::new();
    let postprocessors: HashMap<&str, BookmarkPostprocessorPlugin> = HashMap::new();

    BookmarkPluginRegistry {
        downloaders,
        extractors,
        postprocessors,
    }
}
