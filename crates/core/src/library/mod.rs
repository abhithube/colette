pub use library_repository::*;
pub use library_service::*;

use crate::{Collection, Feed, Folder};

mod library_repository;
mod library_service;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum FeedTreeItem {
    Folder(Folder),
    Feed(Feed),
}

#[derive(Debug, Clone)]
pub enum CollectionTreeItem {
    Folder(Folder),
    Collection(Collection),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
