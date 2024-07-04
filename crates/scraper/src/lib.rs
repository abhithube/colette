mod downloader;
mod extractor;
mod postprocessor;

pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, ExtractorOptions};
pub use downloader::DefaultDownloader;
pub use extractor::DefaultFeedExtractor;
pub use postprocessor::DefaultFeedPostprocessor;
