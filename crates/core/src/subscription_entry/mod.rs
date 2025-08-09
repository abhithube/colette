use chrono::{DateTime, Utc};
pub use get_subscription_entry_handler::*;
pub use list_subscription_entries_handler::*;
pub use mark_subscription_entry_as_read_handler::*;
pub use mark_subscription_entry_as_unread_handler::*;
pub use subscription_entry_repository::*;
use uuid::Uuid;

use crate::{
    FeedEntry,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    pagination::Cursor,
};

mod get_subscription_entry_handler;
mod list_subscription_entries_handler;
mod mark_subscription_entry_as_read_handler;
mod mark_subscription_entry_as_unread_handler;
mod subscription_entry_repository;

#[derive(Debug, Clone)]
pub struct SubscriptionEntry {
    pub id: Uuid,
    pub has_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub subscription_id: Uuid,
    pub feed_entry_id: Uuid,
    pub feed_entry: FeedEntry,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionEntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
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
