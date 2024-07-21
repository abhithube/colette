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
    let mut downloaders = HashMap::from([
        ("www.youtube.com", youtube::DOWNLOADER_PLUGIN),
        ("www.reddit.com", reddit::DOWNLOADER_PLUGIN),
    ]);
    let mut extractors: HashMap<&str, ExtractorPlugin<FeedExtractorOptions, ExtractedFeed>> =
        HashMap::new();
    let mut postprocessors: HashMap<&str, PostprocessorPlugin<ExtractedFeed, (), ProcessedFeed>> =
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

pub fn register_bookmark_plugins<'a>(
) -> PluginRegistry<BookmarkExtractorOptions<'a>, ExtractedBookmark, (), ProcessedBookmark> {
    let mut downloaders: HashMap<&str, DownloaderPlugin> = HashMap::new();
    let mut extractors: HashMap<
        &str,
        ExtractorPlugin<BookmarkExtractorOptions, ExtractedBookmark>,
    > = HashMap::new();
    let mut postprocessors: HashMap<
        &str,
        PostprocessorPlugin<ExtractedBookmark, (), ProcessedBookmark>,
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
