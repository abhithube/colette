use chrono::{DateTime, Utc};
use colette_util::uuid_generate_ts;
use uuid::Uuid;

use crate::{auth::UserId, bookmark::BookmarkFilter};

pub const COLLECTION_TITLE_MAX_LENGTH: usize = 50;

#[derive(Debug, Clone)]
pub struct Collection {
    id: CollectionId,
    title: CollectionTitle,
    filter: BookmarkFilter,
    user_id: UserId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(title: CollectionTitle, filter: BookmarkFilter, user_id: UserId) -> Self {
        let now = Utc::now();

        Self {
            id: uuid_generate_ts(now).into(),
            title,
            filter,
            user_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> CollectionId {
        self.id
    }

    pub fn title(&self) -> &CollectionTitle {
        &self.title
    }

    pub fn set_title(&mut self, value: CollectionTitle) {
        if value != self.title {
            self.title = value;
            self.updated_at = Utc::now();
        }
    }

    pub fn filter(&self) -> &BookmarkFilter {
        &self.filter
    }

    pub fn set_filter(&mut self, value: BookmarkFilter) {
        if value != self.filter {
            self.filter = value;
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

    #[allow(clippy::too_many_arguments)]
    pub fn from_unchecked(
        id: Uuid,
        title: String,
        filter: BookmarkFilter,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: CollectionId(id),
            title: CollectionTitle(title),
            filter,
            user_id: user_id.into(),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionTitle(String);

impl CollectionTitle {
    pub fn new(value: String) -> Result<Self, CollectionError> {
        if value.is_empty() || value.len() > COLLECTION_TITLE_MAX_LENGTH {
            return Err(CollectionError::InvalidTitleLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
    #[error("title must be between 1 and {COLLECTION_TITLE_MAX_LENGTH} characters long")]
    InvalidTitleLength,

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),
}
