use colette_core::subscription_entry;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    ApiState,
    common::{BooleanOp, DateOp, Paginated, TextOp},
    feed_entry::FeedEntry,
};

mod list_feed_entries;

pub const SUBSCRIPTION_ENTRIES_TAG: &str = "Subscription Entries";

#[derive(OpenApi)]
#[openapi(components(schemas(SubscriptionEntry, Paginated<SubscriptionEntry>, SubscriptionEntryFilter, SubscriptionEntryTextField, SubscriptionEntryBooleanField, SubscriptionEntryDateField)))]
pub struct SubscriptionEntryApi;

impl SubscriptionEntryApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(SubscriptionEntryApi::openapi())
            .routes(routes!(list_feed_entries::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionEntry {
    pub entry_id: Uuid,
    pub subscription_id: Uuid,
    pub entry: Option<FeedEntry>,
    pub has_read: Option<bool>,
}

impl From<colette_core::SubscriptionEntry> for SubscriptionEntry {
    fn from(value: colette_core::SubscriptionEntry) -> Self {
        Self {
            entry_id: value.entry_id,
            entry: value.entry.map(Into::into),
            has_read: value.has_read,
            subscription_id: value.subscription_id,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub enum SubscriptionEntryFilter {
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
pub enum SubscriptionEntryTextField {
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
pub enum SubscriptionEntryBooleanField {
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
pub enum SubscriptionEntryDateField {
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
