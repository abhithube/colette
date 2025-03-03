use chrono::{DateTime, Utc};
use colette_core::feed_entry;
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    ApiState,
    common::{BooleanOp, DateOp, Paginated, TextOp},
};

mod get_feed_entry;
mod list_feed_entries;
mod update_feed_entry;

pub const FEED_ENTRIES_TAG: &str = "Feed Entries";

#[derive(OpenApi)]
#[openapi(components(schemas(FeedEntry, Paginated<FeedEntry>, update_feed_entry::FeedEntryUpdate, FeedEntryFilter, FeedEntryTextField, FeedEntryBooleanField, FeedEntryDateField)))]
pub struct FeedEntryApi;

impl FeedEntryApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(FeedEntryApi::openapi())
            .routes(routes!(list_feed_entries::handler))
            .routes(routes!(get_feed_entry::handler, update_feed_entry::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedEntry {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    #[schema(required)]
    pub description: Option<String>,
    #[schema(required)]
    pub author: Option<String>,
    #[schema(required)]
    pub thumbnail_url: Option<Url>,
    pub has_read: bool,
    pub feed_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<colette_core::FeedEntry> for FeedEntry {
    fn from(value: colette_core::FeedEntry) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            has_read: value.has_read,
            feed_id: value.feed_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub enum FeedEntryFilter {
    Text {
        field: FeedEntryTextField,
        op: TextOp,
    },
    Boolean {
        field: FeedEntryBooleanField,
        op: BooleanOp,
    },
    Date {
        field: FeedEntryDateField,
        op: DateOp,
    },

    And(Vec<FeedEntryFilter>),
    Or(Vec<FeedEntryFilter>),
    Not(Box<FeedEntryFilter>),
}

impl From<FeedEntryFilter> for feed_entry::FeedEntryFilter {
    fn from(value: FeedEntryFilter) -> Self {
        match value {
            FeedEntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            FeedEntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            FeedEntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            FeedEntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            FeedEntryFilter::Or(filters) => Self::Or(filters.into_iter().map(Into::into).collect()),
            FeedEntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

impl From<feed_entry::FeedEntryFilter> for FeedEntryFilter {
    fn from(value: feed_entry::FeedEntryFilter) -> Self {
        match value {
            feed_entry::FeedEntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            feed_entry::FeedEntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            feed_entry::FeedEntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            feed_entry::FeedEntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            feed_entry::FeedEntryFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            feed_entry::FeedEntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryTextField {
    Link,
    Title,
    Description,
    Author,
    Tag,
}

impl From<FeedEntryTextField> for feed_entry::FeedEntryTextField {
    fn from(value: FeedEntryTextField) -> Self {
        match value {
            FeedEntryTextField::Title => Self::Title,
            FeedEntryTextField::Link => Self::Link,
            FeedEntryTextField::Description => Self::Description,
            FeedEntryTextField::Author => Self::Author,
            FeedEntryTextField::Tag => Self::Tag,
        }
    }
}

impl From<feed_entry::FeedEntryTextField> for FeedEntryTextField {
    fn from(value: feed_entry::FeedEntryTextField) -> Self {
        match value {
            feed_entry::FeedEntryTextField::Title => Self::Title,
            feed_entry::FeedEntryTextField::Link => Self::Link,
            feed_entry::FeedEntryTextField::Description => Self::Description,
            feed_entry::FeedEntryTextField::Author => Self::Author,
            feed_entry::FeedEntryTextField::Tag => Self::Tag,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryBooleanField {
    HasRead,
}

impl From<FeedEntryBooleanField> for feed_entry::FeedEntryBooleanField {
    fn from(value: FeedEntryBooleanField) -> Self {
        match value {
            FeedEntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

impl From<feed_entry::FeedEntryBooleanField> for FeedEntryBooleanField {
    fn from(value: feed_entry::FeedEntryBooleanField) -> Self {
        match value {
            feed_entry::FeedEntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryDateField {
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

impl From<FeedEntryDateField> for feed_entry::FeedEntryDateField {
    fn from(value: FeedEntryDateField) -> Self {
        match value {
            FeedEntryDateField::PublishedAt => Self::PublishedAt,
            FeedEntryDateField::CreatedAt => Self::CreatedAt,
            FeedEntryDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}

impl From<feed_entry::FeedEntryDateField> for FeedEntryDateField {
    fn from(value: feed_entry::FeedEntryDateField) -> Self {
        match value {
            feed_entry::FeedEntryDateField::PublishedAt => Self::PublishedAt,
            feed_entry::FeedEntryDateField::CreatedAt => Self::CreatedAt,
            feed_entry::FeedEntryDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}
