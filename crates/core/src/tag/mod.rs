use chrono::{DateTime, Utc};
pub use create_tag_handler::*;
pub use delete_tag_handler::*;
pub use get_tag_handler::*;
pub use list_tags_handler::*;
pub use tag_repository::*;
pub use update_tag_handler::*;
use uuid::Uuid;

use crate::pagination::Cursor;

mod create_tag_handler;
mod delete_tag_handler;
mod get_tag_handler;
mod list_tags_handler;
mod tag_repository;
mod update_tag_handler;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing)]
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TagCursor {
    pub title: String,
}

impl Cursor for Tag {
    type Data = TagCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}
