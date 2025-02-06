pub use scraper_repository::*;
pub use scraper_service::*;

use crate::{bookmark, feed};

mod scraper_repository;
mod scraper_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error(transparent)]
    Feed(#[from] feed::ScraperError),

    #[error(transparent)]
    Bookmark(#[from] bookmark::ScraperError),
}
