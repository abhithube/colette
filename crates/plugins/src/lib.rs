use std::collections::HashMap;

use colette_scraper::{
    downloader::DownloaderPlugin, BookmarkExtractorPlugin, BookmarkPluginRegistry,
    BookmarkPostprocessorPlugin, FeedDetectorPlugin, FeedExtractorPlugin, FeedPluginRegistry,
    FeedPostprocessorPlugin,
};
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<'a>() -> FeedPluginRegistry<'a> {
    let downloaders = HashMap::from([
        ("www.youtube.com", youtube::DOWNLOADER_PLUGIN),
        ("www.reddit.com", reddit::DOWNLOADER_PLUGIN),
    ]);
    let detectors: HashMap<&str, FeedDetectorPlugin> = HashMap::new();
    let extractors: HashMap<&str, FeedExtractorPlugin> = HashMap::new();
    let postprocessors: HashMap<&str, FeedPostprocessorPlugin> = HashMap::new();

    FeedPluginRegistry {
        downloaders,
        detectors,
        extractors,
        postprocessors,
    }
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
