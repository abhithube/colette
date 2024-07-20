use std::{collections::HashMap, sync::Arc};

use colette_core::{
    bookmarks::{ExtractedBookmark, ProcessedBookmark},
    feeds::{ExtractedFeed, ProcessedFeed},
    utils::scraper::{Downloader, Extractor, PluginRegistry, Postprocessor},
};
#[allow(unused_imports)]
use custom::*;
use reddit::RedditFeedPlugin;
use youtube::YouTubeFeedPlugin;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins(
    downloader: Arc<dyn Downloader>,
    _extractor: Arc<dyn Extractor<ExtractedFeed>>,
    _postprocessor: Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed>>,
) -> PluginRegistry<ExtractedFeed, ProcessedFeed> {
    let yt_feed_plugin = Arc::new(YouTubeFeedPlugin::new(downloader.clone()));
    let reddit_feed_plugin = Arc::new(RedditFeedPlugin::new(downloader.clone()));

    let mut downloaders = HashMap::from([
        ("www.youtube.com", yt_feed_plugin as Arc<dyn Downloader>),
        ("www.reddit.com", reddit_feed_plugin),
    ]);
    let mut extractors: HashMap<&str, Arc<dyn Extractor<ExtractedFeed>>> = HashMap::new();
    let mut postprocessors: HashMap<&str, Arc<dyn Postprocessor<ExtractedFeed, ProcessedFeed>>> =
        HashMap::new();

    downloaders.extend(HashMap::new());
    extractors.extend(HashMap::new());
    postprocessors.extend(HashMap::new());

    PluginRegistry {
        downloaders,
        extractors,
        postprocessors,
    }
}

pub fn register_bookmark_plugins(
    _downloader: Arc<dyn Downloader>,
    _extractor: Arc<dyn Extractor<ExtractedBookmark>>,
    _postprocessor: Arc<dyn Postprocessor<ExtractedBookmark, ProcessedBookmark>>,
) -> PluginRegistry<ExtractedBookmark, ProcessedBookmark> {
    let mut downloaders: HashMap<&str, Arc<dyn Downloader>> = HashMap::new();
    let mut extractors: HashMap<&str, Arc<dyn Extractor<ExtractedBookmark>>> = HashMap::new();
    let mut postprocessors: HashMap<
        &str,
        Arc<dyn Postprocessor<ExtractedBookmark, ProcessedBookmark>>,
    > = HashMap::new();

    downloaders.extend(HashMap::new());
    extractors.extend(HashMap::new());
    postprocessors.extend(HashMap::new());

    PluginRegistry {
        downloaders,
        extractors,
        postprocessors,
    }
}
