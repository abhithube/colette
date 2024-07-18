pub use bookmarks::{BookmarkScraper, DefaultBookmarkExtractor, DefaultBookmarkPostprocessor};
pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions};
pub use downloader::DefaultDownloader;
pub use feeds::{
    atom_extractor_options, dublin_core_extractor_options, itunes_extractor_options,
    media_extractor_options, rss_extractor_options, DefaultFeedExtractor, DefaultFeedPostprocessor,
    FeedScraper,
};
pub use registry::PluginRegistry;

mod bookmarks;
mod downloader;
mod feeds;
mod registry;
mod utils;
