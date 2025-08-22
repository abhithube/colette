use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{auth::UserId, common::UuidGenerator, pagination::Cursor};

pub const TAG_TITLE_MAX_LENGTH: usize = 50;

#[derive(Debug, Clone)]
pub struct TagDto {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    id: TagId,
    title: TagTitle,
    user_id: UserId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(title: TagTitle, user_id: UserId) -> Self {
        let now = Utc::now();

        Self {
            id: UuidGenerator::new().with_timestamp(now).generate().into(),
            title,
            user_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> TagId {
        self.id
    }

    pub fn title(&self) -> &TagTitle {
        &self.title
    }

    pub fn set_title(&mut self, value: TagTitle) {
        if value != self.title {
            self.title = value;
            self.updated_at = Utc::now();
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn from_unchecked(
        id: Uuid,
        title: String,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: TagId(id),
            title: TagTitle(title),
            user_id: user_id.into(),
            created_at,
            updated_at,
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagTitle(String);

impl TagTitle {
    pub fn new(value: String) -> Result<Self, TagError> {
        if value.is_empty() || value.len() > TAG_TITLE_MAX_LENGTH {
            return Err(TagError::InvalidTitleLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TagCursor {
    pub title: String,
}

impl Cursor for TagDto {
    type Data = TagCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("title must be between 1 and {TAG_TITLE_MAX_LENGTH} characters long")]
    InvalidTitleLength,

    #[error("tag already exists with title: {0}")]
    Conflict(TagTitle),

    #[error("tag not found with ID: {0}")]
    NotFound(TagId),
}
