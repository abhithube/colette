use std::collections::HashMap;

use colette_core::{
    bookmarks::{BookmarkExtractorOptions, ExtractedBookmark, ProcessedBookmark},
    feeds::{ExtractedFeed, ProcessedFeed},
    utils::scraper::{DownloaderPlugin, ExtractorPlugin, PluginRegistry, PostprocessorPlugin},
};
use colette_scraper::FeedExtractorOptions;
#[allow(unused_imports)]
use custom::*;

mod custom;
mod reddit;
mod youtube;

pub fn register_feed_plugins<'a>(
) -> PluginRegistry<FeedExtractorOptions<'a>, ExtractedFeed, (), ProcessedFeed> {
    let downloaders = HashMap::from([
        ("www.youtube.com", youtube::DOWNLOADER_PLUGIN),
        ("www.reddit.com", reddit::DOWNLOADER_PLUGIN),
    ]);
    let extractors: HashMap<&str, ExtractorPlugin<FeedExtractorOptions, ExtractedFeed>> =
        HashMap::new();
    let postprocessors: HashMap<&str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>> =
        HashMap::new();

    PluginRegistry {
        downloaders,
        extractors,
        postprocessors,
    }
}

pub fn register_bookmark_plugins<'a>(
) -> PluginRegistry<BookmarkExtractorOptions<'a>, ExtractedBookmark, (), ProcessedBookmark> {
    let downloaders: HashMap<&str, DownloaderPlugin> = HashMap::new();
    let extractors: HashMap<&str, ExtractorPlugin<BookmarkExtractorOptions, ExtractedBookmark>> =
        HashMap::new();
    let postprocessors: HashMap<
        &str,
        PostprocessorPlugin<ExtractedBookmark, (), ProcessedBookmark>,
    > = HashMap::new();

    PluginRegistry {
        downloaders,
        extractors,
        postprocessors,
    }
}
