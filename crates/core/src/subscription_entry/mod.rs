use chrono::{DateTime, Utc};
use colette_util::base64;
use sea_orm::DbErr;
pub use subscription_entry_repository::*;
pub use subscription_entry_service::*;
use uuid::Uuid;

use crate::{
    FeedEntry,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
    stream,
};

mod subscription_entry_repository;
mod subscription_entry_service;

#[derive(Debug, Clone)]
pub struct SubscriptionEntry {
    pub entry: FeedEntry,
    pub has_read: bool,
    pub subscription_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub id: Uuid,
    pub published_at: DateTime<Utc>,
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
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Stream(#[from] stream::Error),

    #[error(transparent)]
    Database(#[from] DbErr),
}
