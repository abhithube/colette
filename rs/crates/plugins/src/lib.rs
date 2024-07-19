use std::{collections::HashMap, sync::Arc};

use colette_core::{
    bookmarks::{ExtractedBookmark, ProcessedBookmark},
    feeds::{ExtractedFeed, ProcessedFeed},
    utils::scraper::{Downloader, Extractor, PluginRegistry, Postprocessor},
};
use reddit::RedditFeedPlugin;
use youtube::YouTubeFeedPlugin;

mod reddit;
mod youtube;

pub fn register_feed_plugins(
    downloader: Arc<dyn Downloader + Send + Sync>,
    _extractor: Arc<dyn Extractor<ExtractedFeed> + Send + Sync>,
    _postprocessor: Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed> + Send + Sync>,
) -> PluginRegistry<ExtractedFeed, ProcessedFeed> {
    let yt_feed_plugin = Arc::new(YouTubeFeedPlugin::new(downloader.clone()));
    let reddit_feed_plugin = Arc::new(RedditFeedPlugin::new(downloader.clone()));

    PluginRegistry {
        downloaders: HashMap::from([
            (
                "www.youtube.com",
                yt_feed_plugin as Arc<dyn Downloader + Send + Sync>,
            ),
            ("www.reddit.com", reddit_feed_plugin),
        ]),
        extractors: HashMap::new(),
        postprocessors: HashMap::new(),
    }
}

pub fn register_bookmark_plugins(
    _downloader: Arc<dyn Downloader + Send + Sync>,
    _extractor: Arc<dyn Extractor<ExtractedBookmark> + Send + Sync>,
    _postprocessor: Arc<dyn Postprocessor<ExtractedBookmark, ProcessedBookmark> + Send + Sync>,
) -> PluginRegistry<ExtractedBookmark, ProcessedBookmark> {
    PluginRegistry {
        downloaders: HashMap::new(),
        extractors: HashMap::new(),
        postprocessors: HashMap::new(),
    }
}
