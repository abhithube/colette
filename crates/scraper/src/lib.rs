mod downloader;
mod extractor;
mod options;
mod postprocessor;

pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, ExtractorOptions};
pub use downloader::DefaultDownloader;
pub use extractor::DefaultFeedExtractor;
pub use options::{AtomExtractorOptions, RssExtractorOptions};
pub use postprocessor::DefaultFeedPostprocessor;
