use std::collections::HashMap;

use colette_scraper::{
    detector::DetectorPlugin, downloader::DownloaderPlugin, extractor::ExtractorPlugin,
    postprocessor::PostprocessorPlugin, BookmarkExtractorOptions, BookmarkPluginRegistry,
    ExtractedBookmark, ExtractedFeed, FeedExtractorOptions, FeedPluginRegistry, ProcessedBookmark,
    ProcessedFeed,
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
    let detectors: HashMap<&str, DetectorPlugin> = HashMap::new();
    let extractors: HashMap<&str, ExtractorPlugin<FeedExtractorOptions, ExtractedFeed>> =
        HashMap::new();
    let postprocessors: HashMap<&str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>> =
        HashMap::new();

    FeedPluginRegistry {
        downloaders,
        detectors,
        extractors,
        postprocessors,
    }
}

pub fn register_bookmark_plugins<'a>() -> BookmarkPluginRegistry<'a> {
    let downloaders: HashMap<&str, DownloaderPlugin> = HashMap::new();
    let extractors: HashMap<&str, ExtractorPlugin<BookmarkExtractorOptions, ExtractedBookmark>> =
        HashMap::new();
    let postprocessors: HashMap<
        &str,
        PostprocessorPlugin<ExtractedBookmark, (), ProcessedBookmark>,
    > = HashMap::new();

    BookmarkPluginRegistry {
        downloaders,
        extractors,
        postprocessors,
    }
}
