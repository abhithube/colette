pub use bookmark::{
    BookmarkExtractorOptions, BookmarkPluginRegistry, BookmarkScraper, ExtractedBookmark,
    ProcessedBookmark,
};
pub use feed::{
    ExtractedFeed, ExtractedFeedEntry, FeedExtractorOptions, FeedPluginRegistry, FeedScraper,
    ProcessedFeed, ProcessedFeedEntry,
};

mod bookmark;
mod feed;
pub mod utils;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Download(#[from] DownloaderError),

    #[error("failed to parse document")]
    Parse,

    #[error(transparent)]
    Extract(#[from] ExtractorError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessorError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DownloaderError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct ExtractorError(#[from] pub anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PostprocessorError(#[from] pub anyhow::Error);
