pub use bookmarks::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options, DefaultBookmarkExtractor, DefaultBookmarkPostprocessor,
    DefaultBookmarkScraper,
};
pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions};
pub use downloader::DefaultDownloader;
pub use feeds::{DefaultFeedPostprocessor, DefaultFeedScraper, HtmlExtractor};

mod bookmarks;
mod downloader;
mod feeds;
