pub use colette_scraper::feed::ProcessedFeed;
pub use scraper_repository::*;
pub use scraper_service::*;

mod scraper_repository;
mod scraper_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
