mod downloader;
mod postprocessor;

pub use colette_core::feeds::{ExtractedEntry, ExtractedFeed, ExtractorOptions};
pub use downloader::DefaultDownloader;
pub use postprocessor::DefaultFeedPostprocessor;
