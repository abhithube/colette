pub use library_repository::*;
pub use library_service::*;

use crate::{Bookmark, Feed, Folder};

mod library_repository;
mod library_service;

#[derive(Debug, Clone)]
pub enum LibraryItem {
    Folder(Folder),
    Feed(Feed),
    Bookmark(Bookmark),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
