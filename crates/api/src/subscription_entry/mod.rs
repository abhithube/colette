use chrono::{DateTime, Utc};
use colette_core::subscription_entry;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    ApiState,
    common::{BooleanOp, DateOp, Paginated, TextOp},
    feed_entry::FeedEntry,
};

mod list_subscription_entries;

const SUBSCRIPTION_ENTRIES_TAG: &str = "Subscription Entries";

#[derive(OpenApi)]
#[openapi(components(schemas(SubscriptionEntry, SubscriptionEntryDetails, Paginated<SubscriptionEntryDetails>, SubscriptionEntryFilter, SubscriptionEntryTextField, SubscriptionEntryBooleanField, SubscriptionEntryDateField)))]
pub(crate) struct SubscriptionEntryApi;

impl SubscriptionEntryApi {
    pub(crate) fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(SubscriptionEntryApi::openapi())
            .routes(routes!(list_subscription_entries::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionEntry {
    subscription_id: Uuid,
    feed_entry_id: Uuid,
    has_read: bool,
    read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct SubscriptionEntryDetails {
    subscription_entry: SubscriptionEntry,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed_entry: Option<FeedEntry>,
}

impl From<colette_core::SubscriptionEntry> for SubscriptionEntry {
    fn from(value: colette_core::SubscriptionEntry) -> Self {
        Self {
            subscription_id: value.subscription_id,
            feed_entry_id: value.feed_entry_id,
            has_read: value.has_read.unwrap_or_default(),
            read_at: value.read_at,
        }
    }
}

impl From<colette_core::SubscriptionEntry> for SubscriptionEntryDetails {
    fn from(value: colette_core::SubscriptionEntry) -> Self {
        let feed_entry = value.feed_entry.clone().map(Into::into);

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
