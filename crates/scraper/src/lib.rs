pub use bookmark::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options, DefaultBookmarkExtractor, DefaultBookmarkPostprocessor,
    DefaultBookmarkScraper,
};
pub use colette_core::feed::{ExtractedFeed, ExtractedFeedEntry, FeedExtractorOptions};
pub use downloader::DefaultDownloader;
pub use feed::{DefaultFeedPostprocessor, DefaultFeedScraper, HtmlExtractor};

mod bookmark;
mod downloader;
mod feed;
mod utils;
