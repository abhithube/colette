mod error;
mod model;
mod repository;
mod service;

pub use error::Error;
pub use model::{
    CreateFeedDto, ExtractedEntry, ExtractedFeed, ExtractorOptions, Feed, ProcessedEntry,
    ProcessedFeed,
};
pub use repository::{FeedCreateData, FeedFindManyParams, FeedsRepository};
pub use service::FeedsService;
