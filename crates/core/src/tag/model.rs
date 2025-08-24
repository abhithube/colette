use chrono::{DateTime, Utc};
use colette_util::uuid_generate_ts;
use uuid::Uuid;

use crate::auth::UserId;

pub const TAG_TITLE_MAX_LENGTH: usize = 50;

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
            id: uuid_generate_ts(now).into(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("title must be between 1 and {TAG_TITLE_MAX_LENGTH} characters long")]
    InvalidTitleLength,

    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),
}
