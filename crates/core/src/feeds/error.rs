use thiserror::Error;

use crate::scraper;

#[derive(Debug, Error)]
pub enum Error {
    #[error("feed not found with id: {0}")]
    NotFound(String),

    #[error(transparent)]
    Scraper(#[from] scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
