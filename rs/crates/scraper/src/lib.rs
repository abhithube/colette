pub use bookmarks::{BookmarkScraper, DefaultBookmarkExtractor, DefaultBookmarkPostprocessor};
pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions};
pub use downloader::DefaultDownloader;
pub use feeds::{
    AtomExtractorOptions, DefaultFeedExtractor, DefaultFeedPostprocessor, FeedScraper,
    RssExtractorOptions,
};
pub use registry::PluginRegistry;

mod bookmarks;
mod downloader;
mod feeds;
mod registry;
mod utils;
