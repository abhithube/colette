pub use archive_thumbnail::*;
pub use import_bookmarks::*;
pub use refresh_feeds::*;
pub use scrape_bookmark::*;
pub use scrape_feed::*;

mod archive_thumbnail;
mod import_bookmarks;
mod refresh_feeds;
mod scrape_bookmark;
mod scrape_feed;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),

    #[error("service error: {0}")]
    Service(String),
}
