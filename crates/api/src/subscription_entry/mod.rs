use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::subscription_entry;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    ApiState,
    common::{BooleanOp, DateOp, TextOp},
    feed_entry::FeedEntry,
    pagination::Paginated,
};

mod list_subscription_entries;
mod mark_subscription_entry_as_read;
mod mark_subscription_entry_as_unread;

const SUBSCRIPTION_ENTRIES_TAG: &str = "Subscription Entries";

#[derive(OpenApi)]
#[openapi(
    components(schemas(SubscriptionEntry, SubscriptionEntryDetails, Paginated<SubscriptionEntryDetails>, SubscriptionEntryFilter, SubscriptionEntryTextField, SubscriptionEntryBooleanField, SubscriptionEntryDateField)),
    paths(list_subscription_entries::handler, mark_subscription_entry_as_read::handler, mark_subscription_entry_as_unread::handler)
)]
pub(crate) struct SubscriptionEntryApi;

impl SubscriptionEntryApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_subscription_entries::handler))
            .route(
                "/{id}/markAsRead",
                routing::post(mark_subscription_entry_as_read::handler),
            )
            .route(
                "/{id}/markAsUnread",
                routing::post(mark_subscription_entry_as_unread::handler),
            )
    }
}

/// Association of a RSS feed entry to a user subscription. The pairing of subscription ID and feed entry ID is unique.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionEntry {
    /// Unique identifier of the subscription entry
    id: Uuid,
    /// Whether the subscription entry has been marked as read
    has_read: bool,
    /// Timestamp at which the subscription entry has been marked as read
    read_at: Option<DateTime<Utc>>,
    /// Unique identifier of the associated subscription
    subscription_id: Uuid,
    /// Unique identifier of the associated feed entry
    feed_entry_id: Uuid,
}

/// Extended details of a subscription entry
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct SubscriptionEntryDetails {
    /// Subscription entry itself, always present
    subscription_entry: SubscriptionEntry,
    /// Associated feed entry, always present
    feed_entry: FeedEntry,
}

impl From<colette_core::SubscriptionEntry> for SubscriptionEntry {
    fn from(value: colette_core::SubscriptionEntry) -> Self {
        Self {
            id: value.id,
            has_read: value.has_read,
            read_at: value.read_at,
            subscription_id: value.subscription_id,
            feed_entry_id: value.feed_entry_id,
        }
    }
}

impl From<colette_core::SubscriptionEntry> for SubscriptionEntryDetails {
    fn from(value: colette_core::SubscriptionEntry) -> Self {
        let feed_entry = value.feed_entry.clone().into();

        Self {
            subscription_entry: value.into(),
            feed_entry,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub(crate) enum SubscriptionEntryFilter {
    Text {
        field: SubscriptionEntryTextField,
        op: TextOp,
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

impl From<SubscriptionEntryFilter> for subscription_entry::SubscriptionEntryFilter {
    fn from(value: SubscriptionEntryFilter) -> Self {
        match value {
            SubscriptionEntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            SubscriptionEntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            SubscriptionEntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            SubscriptionEntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            SubscriptionEntryFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            SubscriptionEntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

impl From<subscription_entry::SubscriptionEntryFilter> for SubscriptionEntryFilter {
    fn from(value: subscription_entry::SubscriptionEntryFilter) -> Self {
        match value {
            subscription_entry::SubscriptionEntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            subscription_entry::SubscriptionEntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            subscription_entry::SubscriptionEntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            subscription_entry::SubscriptionEntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            subscription_entry::SubscriptionEntryFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            subscription_entry::SubscriptionEntryFilter::Not(filter) => {
                Self::Not(Box::new((*filter).into()))
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SubscriptionEntryTextField {
    Link,
    Title,
    Description,
    Author,
    Tag,
}

impl From<SubscriptionEntryTextField> for subscription_entry::SubscriptionEntryTextField {
    fn from(value: SubscriptionEntryTextField) -> Self {
        match value {
            SubscriptionEntryTextField::Title => Self::Title,
            SubscriptionEntryTextField::Link => Self::Link,
            SubscriptionEntryTextField::Description => Self::Description,
            SubscriptionEntryTextField::Author => Self::Author,
            SubscriptionEntryTextField::Tag => Self::Tag,
        }
    }
}

impl From<subscription_entry::SubscriptionEntryTextField> for SubscriptionEntryTextField {
    fn from(value: subscription_entry::SubscriptionEntryTextField) -> Self {
        match value {
            subscription_entry::SubscriptionEntryTextField::Title => Self::Title,
            subscription_entry::SubscriptionEntryTextField::Link => Self::Link,
            subscription_entry::SubscriptionEntryTextField::Description => Self::Description,
            subscription_entry::SubscriptionEntryTextField::Author => Self::Author,
            subscription_entry::SubscriptionEntryTextField::Tag => Self::Tag,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SubscriptionEntryBooleanField {
    HasRead,
}

impl From<SubscriptionEntryBooleanField> for subscription_entry::SubscriptionEntryBooleanField {
    fn from(value: SubscriptionEntryBooleanField) -> Self {
        match value {
            SubscriptionEntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

impl From<subscription_entry::SubscriptionEntryBooleanField> for SubscriptionEntryBooleanField {
    fn from(value: subscription_entry::SubscriptionEntryBooleanField) -> Self {
        match value {
            subscription_entry::SubscriptionEntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SubscriptionEntryDateField {
    PublishedAt,
}

impl From<SubscriptionEntryDateField> for subscription_entry::SubscriptionEntryDateField {
    fn from(value: SubscriptionEntryDateField) -> Self {
        match value {
            SubscriptionEntryDateField::PublishedAt => Self::PublishedAt,
        }
    }
}

impl From<subscription_entry::SubscriptionEntryDateField> for SubscriptionEntryDateField {
    fn from(value: subscription_entry::SubscriptionEntryDateField) -> Self {
        match value {
            subscription_entry::SubscriptionEntryDateField::PublishedAt => Self::PublishedAt,
        }
    }
}
