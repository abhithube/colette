use chrono::{DateTime, Utc};
pub use collection_repository::*;
pub use create_collection_handler::*;
pub use delete_collection_handler::*;
pub use get_collection_handler::*;
pub use list_collections_handler::*;
pub use update_collection_handler::*;
use uuid::Uuid;

use crate::{bookmark::BookmarkFilter, pagination::Cursor};

mod collection_repository;
mod create_collection_handler;
mod delete_collection_handler;
mod get_collection_handler;
mod list_collections_handler;
mod update_collection_handler;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CollectionCursor {
    pub title: String,
}

impl Cursor for Collection {
    type Data = CollectionCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}
