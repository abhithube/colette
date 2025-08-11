use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{pagination::Cursor, user::UserId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: TagId,
    pub title: String,
    #[serde(skip_serializing)]
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    pub fn authorize(&self, user_id: UserId) -> Result<(), TagError> {
        if self.user_id != user_id {
            return Err(TagError::Forbidden(user_id));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TagId(Uuid);

impl TagId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for TagId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("not authorized to access tag with ID: {0}")]
    Forbidden(UserId),
}
