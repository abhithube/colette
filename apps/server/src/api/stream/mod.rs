use chrono::{DateTime, Utc};
use colette_core::stream;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    ApiState,
    common::{BooleanOp, DateOp, TextOp},
};
use crate::api::common::Paginated;

mod create_stream;
mod delete_stream;
mod get_stream;
mod list_stream_entries;
mod list_streams;
mod update_stream;

pub const STREAMS_TAG: &str = "Streams";

#[derive(OpenApi)]
#[openapi(components(schemas(Stream, FeedEntryFilter, FeedEntryTextField, FeedEntryBooleanField, FeedEntryDateField, Paginated<Stream>, create_stream::StreamCreate, update_stream::StreamUpdate)))]
pub struct StreamApi;

impl StreamApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(StreamApi::openapi())
            .routes(routes!(list_streams::handler, create_stream::handler))
            .routes(routes!(
                get_stream::handler,
                update_stream::handler,
                delete_stream::handler
            ))
            .routes(routes!(list_stream_entries::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub id: Uuid,
    pub title: String,
    pub filter: FeedEntryFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<colette_core::Stream> for Stream {
    fn from(value: colette_core::Stream) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter.into(),
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

impl From<FeedEntryFilter> for stream::FeedEntryFilter {
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

impl From<stream::FeedEntryFilter> for FeedEntryFilter {
    fn from(value: stream::FeedEntryFilter) -> Self {
        match value {
            stream::FeedEntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            stream::FeedEntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            stream::FeedEntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            stream::FeedEntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            stream::FeedEntryFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            stream::FeedEntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
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

impl From<FeedEntryTextField> for stream::FeedEntryTextField {
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

impl From<stream::FeedEntryTextField> for FeedEntryTextField {
    fn from(value: stream::FeedEntryTextField) -> Self {
        match value {
            stream::FeedEntryTextField::Title => Self::Title,
            stream::FeedEntryTextField::Link => Self::Link,
            stream::FeedEntryTextField::Description => Self::Description,
            stream::FeedEntryTextField::Author => Self::Author,
            stream::FeedEntryTextField::Tag => Self::Tag,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FeedEntryBooleanField {
    HasRead,
}

impl From<FeedEntryBooleanField> for stream::FeedEntryBooleanField {
    fn from(value: FeedEntryBooleanField) -> Self {
        match value {
            FeedEntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

impl From<stream::FeedEntryBooleanField> for FeedEntryBooleanField {
    fn from(value: stream::FeedEntryBooleanField) -> Self {
        match value {
            stream::FeedEntryBooleanField::HasRead => Self::HasRead,
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

impl From<FeedEntryDateField> for stream::FeedEntryDateField {
    fn from(value: FeedEntryDateField) -> Self {
        match value {
            FeedEntryDateField::PublishedAt => Self::PublishedAt,
            FeedEntryDateField::CreatedAt => Self::CreatedAt,
            FeedEntryDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}

impl From<stream::FeedEntryDateField> for FeedEntryDateField {
    fn from(value: stream::FeedEntryDateField) -> Self {
        match value {
            stream::FeedEntryDateField::PublishedAt => Self::PublishedAt,
            stream::FeedEntryDateField::CreatedAt => Self::CreatedAt,
            stream::FeedEntryDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}
