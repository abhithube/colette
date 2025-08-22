use std::fmt;

use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{
    auth::UserId,
    common::UuidGenerator,
    feed::FeedId,
    pagination::Cursor,
    tag::{TagDto, TagId},
};

pub const SUBSCRIPTION_TITLE_MAX_LENGTH: usize = 50;
pub const SUBSCRIPTION_DESCRIPTION_MAX_LENGTH: usize = 500;
pub const SUBSCRIPTION_TAG_MAX_COUNT: usize = 20;

#[derive(Debug, Clone)]
pub struct SubscriptionDto {
    pub id: Uuid,
    pub source_url: Url,
    pub link: Url,
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
    pub tags: Vec<TagDto>,
    pub unread_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    id: SubscriptionId,
    title: SubscriptionTitle,
    description: Option<SubscriptionDescription>,
    feed_id: FeedId,
    tags: Vec<TagId>,
    user_id: UserId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Subscription {
    pub fn new(
        title: SubscriptionTitle,
        description: Option<SubscriptionDescription>,
        feed_id: FeedId,
        user_id: UserId,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: UuidGenerator::new().with_timestamp(now).generate().into(),
            title,
            description,
            feed_id,
            tags: Vec::new(),
            user_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    pub fn title(&self) -> &SubscriptionTitle {
        &self.title
    }

    pub fn set_title(&mut self, value: SubscriptionTitle) {
        if value != self.title {
            self.title = value;
            self.updated_at = Utc::now();
        }
    }

    pub fn description(&self) -> Option<&SubscriptionDescription> {
        self.description.as_ref()
    }

    pub fn set_description(&mut self, value: SubscriptionDescription) {
        if self.description.as_ref().is_none_or(|e| &value != e) {
            self.description = Some(value);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_description(&mut self) {
        if self.description.is_some() {
            self.description = None;
            self.updated_at = Utc::now();
        }
    }

    pub fn feed_id(&self) -> FeedId {
        self.feed_id
    }

    pub fn tags(&self) -> &[TagId] {
        &self.tags
    }

    pub fn set_tags(&mut self, value: Vec<TagId>) -> Result<(), SubscriptionError> {
        if value.len() > SUBSCRIPTION_TAG_MAX_COUNT {
            return Err(SubscriptionError::TooManyTags);
        }

        self.tags = value;

        Ok(())
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
        description: Option<String>,
        feed_id: Uuid,
        tags: Vec<Uuid>,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: SubscriptionId(id),
            title: SubscriptionTitle(title),
            description: description.map(SubscriptionDescription),
            feed_id: feed_id.into(),
            tags: tags.into_iter().map(Into::into).collect(),
            user_id: user_id.into(),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionId(Uuid);

impl SubscriptionId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for SubscriptionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionTitle(String);

impl SubscriptionTitle {
    pub fn new(value: String) -> Result<Self, SubscriptionError> {
        if value.is_empty() || value.len() > SUBSCRIPTION_TITLE_MAX_LENGTH {
            return Err(SubscriptionError::InvalidTitleLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionDescription(String);

impl SubscriptionDescription {
    pub fn new(value: String) -> Result<Self, SubscriptionError> {
        if value.is_empty() || value.len() > SUBSCRIPTION_DESCRIPTION_MAX_LENGTH {
            return Err(SubscriptionError::InvalidDescriptionLength);
        }

        Ok(Self(value))
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionCursor {
    pub title: String,
    pub id: Uuid,
}

impl Cursor for SubscriptionDto {
    type Data = SubscriptionCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
            id: self.id,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubscriptionError {
    #[error("title must be between 1 and {SUBSCRIPTION_TITLE_MAX_LENGTH} characters long")]
    InvalidTitleLength,

    #[error(
        "description must be between 1 and {SUBSCRIPTION_DESCRIPTION_MAX_LENGTH} characters long"
    )]
    InvalidDescriptionLength,

    #[error("subscription cannot have more than {SUBSCRIPTION_TAG_MAX_COUNT} tags")]
    TooManyTags,

    #[error("subscription not found with ID: {0}")]
    NotFound(SubscriptionId),
}
