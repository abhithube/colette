use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{bookmark::BookmarkFilter, pagination::Cursor, auth::UserId};

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionId,
    pub title: String,
    pub filter: BookmarkFilter,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn authorize(&self, user_id: UserId) -> Result<(), CollectionError> {
        if self.user_id != user_id {
            return Err(CollectionError::Forbidden(user_id));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CollectionId(Uuid);

impl CollectionId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for CollectionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for CollectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
    #[error("not authorized to access collection with ID: {0}")]
    Forbidden(UserId),
}
