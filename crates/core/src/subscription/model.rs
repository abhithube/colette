use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Feed, Tag, feed::FeedId, pagination::Cursor, user::UserId};

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub title: String,
    pub description: Option<String>,
    pub feed_id: FeedId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub feed: Feed,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
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

impl Subscription {
    pub fn authorize(&self, user_id: UserId) -> Result<(), SubscriptionError> {
        if self.user_id != user_id {
            return Err(SubscriptionError::Forbidden(user_id));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionCursor {
    pub title: String,
    pub id: SubscriptionId,
}

impl Cursor for Subscription {
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
    #[error("not authorized to access subscription with ID: {0}")]
    Forbidden(UserId),
}
