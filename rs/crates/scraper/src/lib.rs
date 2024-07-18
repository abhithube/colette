pub use bookmarks::BookmarkScraper;
pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions};
pub use downloader::DefaultDownloader;
pub use feeds::{AtomExtractorOptions, DefaultFeedExtractor, FeedScraper, RssExtractorOptions};
pub use postprocessor::DefaultFeedPostprocessor;
pub use registry::PluginRegistry;

mod bookmarks;
mod downloader;
mod feeds;
mod postprocessor;
mod registry;
mod utils;
