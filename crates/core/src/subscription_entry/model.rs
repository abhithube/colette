use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    FeedEntry,
    auth::UserId,
    feed_entry::FeedEntryId,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    pagination::Cursor,
    subscription::SubscriptionId,
};

#[derive(Debug, Clone)]
pub struct SubscriptionEntry {
    pub id: SubscriptionEntryId,
    pub has_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub subscription_id: SubscriptionId,
    pub feed_entry_id: FeedEntryId,
    pub feed_entry: FeedEntry,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionEntryId(Uuid);

impl SubscriptionEntryId {
    pub fn new(id: Uuid) -> Self {
        Into::into(id)
    }

    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for SubscriptionEntryId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl fmt::Display for SubscriptionEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_inner().fmt(f)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionEntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: FeedEntryId,
}

impl Cursor for SubscriptionEntry {
    type Data = SubscriptionEntryCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            published_at: self.feed_entry.published_at,
            id: self.feed_entry.id,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionEntryFilter {
    Text {
        field: SubscriptionEntryTextField,
        op: TextOp,
    },
    Number {
        field: SubscriptionEntryNumberField,
        op: NumberOp,
    },
    Boolean {
        field: SubscriptionEntryBooleanField,
        op: BooleanOp,
    },
    Date {
        field: SubscriptionEntryDateField,
        op: DateOp,
    },

    And(Vec<SubscriptionEntryFilter>),
    Or(Vec<SubscriptionEntryFilter>),
    Not(Box<SubscriptionEntryFilter>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionEntryTextField {
    Link,
    Title,
    Description,
    Author,
    Tag,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionEntryNumberField {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionEntryBooleanField {
    HasRead,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionEntryDateField {
    PublishedAt,
}

#[derive(Debug, thiserror::Error)]
pub enum SubscriptionEntryError {}
