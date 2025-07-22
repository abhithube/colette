use chrono::{DateTime, Utc};
pub use subscription_entry_repository::*;
pub use subscription_entry_service::*;
use uuid::Uuid;

use crate::{
    FeedEntry, collection,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    pagination::Cursor,
};

mod subscription_entry_repository;
mod subscription_entry_service;

#[derive(Debug, Clone)]
pub struct SubscriptionEntry {
    pub subscription_id: Uuid,
    pub feed_entry_id: Uuid,
    pub user_id: Uuid,
    pub feed_entry: Option<FeedEntry>,
    pub has_read: Option<bool>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionEntryCursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
}

impl Cursor for SubscriptionEntry {
    type Data = SubscriptionEntryCursor;

    fn to_cursor(&self) -> Self::Data {
        if let Some(ref feed_entry) = self.feed_entry {
            Self::Data {
                published_at: feed_entry.published_at,
                id: feed_entry.id,
            }
        } else {
            Default::default()
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
pub enum Error {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access feed entry with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Collection(#[from] collection::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
