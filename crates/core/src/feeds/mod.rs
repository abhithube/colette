mod error;
mod model;
mod repository;
mod service;

pub use error::Error;
pub use model::{
    ExtractedEntry, ExtractedFeed, ExtractorOptions, Feed, ProcessedEntry, ProcessedFeed,
};
pub use repository::{FeedCreateData, FeedsRepository};
pub use service::FeedsService;
