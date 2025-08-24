use std::ops::Range;

use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_handler::CollectionDto;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated};

mod create_collection;
mod delete_collection;
mod get_collection;
mod list_collections;
mod update_collection;

const COLLECTIONS_TAG: &str = "Collections";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Collection, Paginated<Collection>, create_collection::CollectionCreate, update_collection::CollectionUpdate, TextOp, BooleanOp, DateOp, BookmarkFilter, BookmarkTextField, BookmarkDateField, EntryFilter, EntryTextField, EntryBooleanField, EntryDateField)),
    paths(list_collections::handler, create_collection::handler, get_collection::handler, update_collection::handler, delete_collection::handler)
)]
pub(crate) struct CollectionApi;

impl CollectionApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_collections::handler))
            .route("/", routing::post(create_collection::handler))
            .route("/{id}", routing::get(get_collection::handler))
            .route("/{id}", routing::patch(update_collection::handler))
            .route("/{id}", routing::delete(delete_collection::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Collection {
    id: Uuid,
    title: String,
    filter: BookmarkFilter,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionDto> for Collection {
    fn from(value: CollectionDto) -> Self {
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
pub(crate) enum TextOp {
    Equals(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

impl From<TextOp> for colette_crud::TextOp {
    fn from(value: TextOp) -> Self {
        match value {
            TextOp::Equals(value) => Self::Equals(value),
            TextOp::Contains(value) => Self::Contains(value),
            TextOp::StartsWith(value) => Self::StartsWith(value),
            TextOp::EndsWith(value) => Self::EndsWith(value),
        }
    }
}

impl From<colette_crud::TextOp> for TextOp {
    fn from(value: colette_crud::TextOp) -> Self {
        match value {
            colette_crud::TextOp::Equals(value) => Self::Equals(value),
            colette_crud::TextOp::Contains(value) => Self::Contains(value),
            colette_crud::TextOp::StartsWith(value) => Self::StartsWith(value),
            colette_crud::TextOp::EndsWith(value) => Self::EndsWith(value),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum NumberOp {
    Equals(f64),
    GreaterThan(f64),
    LessThan(f64),
    Between { start: f64, end: f64 },
}

impl From<NumberOp> for colette_crud::NumberOp {
    fn from(value: NumberOp) -> Self {
        match value {
            NumberOp::Equals(value) => Self::Equals(value),
            NumberOp::GreaterThan(value) => Self::GreaterThan(value),
            NumberOp::LessThan(value) => Self::LessThan(value),
            NumberOp::Between { start, end } => Self::Between(Range { start, end }),
        }
    }
}

impl From<colette_crud::NumberOp> for NumberOp {
    fn from(value: colette_crud::NumberOp) -> Self {
        match value {
            colette_crud::NumberOp::Equals(value) => Self::Equals(value),
            colette_crud::NumberOp::GreaterThan(value) => Self::GreaterThan(value),
            colette_crud::NumberOp::LessThan(value) => Self::LessThan(value),
            colette_crud::NumberOp::Between(value) => Self::Between {
                start: value.start,
                end: value.end,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BooleanOp {
    Equals(bool),
}

impl From<BooleanOp> for colette_crud::BooleanOp {
    fn from(value: BooleanOp) -> Self {
        match value {
            BooleanOp::Equals(value) => Self::Equals(value),
        }
    }
}

impl From<colette_crud::BooleanOp> for BooleanOp {
    fn from(value: colette_crud::BooleanOp) -> Self {
        match value {
            colette_crud::BooleanOp::Equals(value) => Self::Equals(value),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DateOp {
    Before(DateTime<Utc>),
    After(DateTime<Utc>),
    Between {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    InLast(i64),
}

impl From<DateOp> for colette_crud::DateOp {
    fn from(value: DateOp) -> Self {
        match value {
            DateOp::Before(value) => Self::Before(value),
            DateOp::After(value) => Self::After(value),
            DateOp::Between { start, end } => Self::Between(Range { start, end }),
            DateOp::InLast(value) => Self::InLast(value),
        }
    }
}

impl From<colette_crud::DateOp> for DateOp {
    fn from(value: colette_crud::DateOp) -> Self {
        match value {
            colette_crud::DateOp::Before(value) => Self::Before(value),
            colette_crud::DateOp::After(value) => Self::After(value),
            colette_crud::DateOp::Between(value) => Self::Between {
                start: value.start,
                end: value.end,
            },
            colette_crud::DateOp::InLast(value) => Self::InLast(value),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub(crate) enum BookmarkFilter {
    Text {
        field: BookmarkTextField,
        op: TextOp,
    },
    Date {
        field: BookmarkDateField,
        op: DateOp,
    },

    And(Vec<BookmarkFilter>),
    Or(Vec<BookmarkFilter>),
    Not(Box<BookmarkFilter>),
}

impl From<BookmarkFilter> for colette_crud::BookmarkFilter {
    fn from(value: BookmarkFilter) -> Self {
        match value {
            BookmarkFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            BookmarkFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            BookmarkFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            BookmarkFilter::Or(filters) => Self::Or(filters.into_iter().map(Into::into).collect()),
            BookmarkFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

impl From<colette_crud::BookmarkFilter> for BookmarkFilter {
    fn from(value: colette_crud::BookmarkFilter) -> Self {
        match value {
            colette_crud::BookmarkFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            colette_crud::BookmarkFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            colette_crud::BookmarkFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            colette_crud::BookmarkFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            colette_crud::BookmarkFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BookmarkTextField {
    Link,
    Title,
    Author,
    Tag,
}

impl From<BookmarkTextField> for colette_crud::BookmarkTextField {
    fn from(value: BookmarkTextField) -> Self {
        match value {
            BookmarkTextField::Title => Self::Title,
            BookmarkTextField::Link => Self::Link,
            BookmarkTextField::Author => Self::Author,
            BookmarkTextField::Tag => Self::Tag,
        }
    }
}

impl From<colette_crud::BookmarkTextField> for BookmarkTextField {
    fn from(value: colette_crud::BookmarkTextField) -> Self {
        match value {
            colette_crud::BookmarkTextField::Title => Self::Title,
            colette_crud::BookmarkTextField::Link => Self::Link,
            colette_crud::BookmarkTextField::Author => Self::Author,
            colette_crud::BookmarkTextField::Tag => Self::Tag,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BookmarkDateField {
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

impl From<BookmarkDateField> for colette_crud::BookmarkDateField {
    fn from(value: BookmarkDateField) -> Self {
        match value {
            BookmarkDateField::PublishedAt => Self::PublishedAt,
            BookmarkDateField::CreatedAt => Self::CreatedAt,
            BookmarkDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}

impl From<colette_crud::BookmarkDateField> for BookmarkDateField {
    fn from(value: colette_crud::BookmarkDateField) -> Self {
        match value {
            colette_crud::BookmarkDateField::PublishedAt => Self::PublishedAt,
            colette_crud::BookmarkDateField::CreatedAt => Self::CreatedAt,
            colette_crud::BookmarkDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub(crate) enum EntryFilter {
    Text {
        field: EntryTextField,
        op: TextOp,
    },
    Boolean {
        field: EntryBooleanField,
        op: BooleanOp,
    },
    Date {
        field: EntryDateField,
        op: DateOp,
    },

    And(Vec<EntryFilter>),
    Or(Vec<EntryFilter>),
    Not(Box<EntryFilter>),
}

impl From<EntryFilter> for colette_crud::EntryFilter {
    fn from(value: EntryFilter) -> Self {
        match value {
            EntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            EntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            EntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            EntryFilter::And(filters) => Self::And(filters.into_iter().map(Into::into).collect()),
            EntryFilter::Or(filters) => Self::Or(filters.into_iter().map(Into::into).collect()),
            EntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

impl From<colette_crud::EntryFilter> for EntryFilter {
    fn from(value: colette_crud::EntryFilter) -> Self {
        match value {
            colette_crud::EntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            colette_crud::EntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            colette_crud::EntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            colette_crud::EntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            colette_crud::EntryFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            colette_crud::EntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EntryTextField {
    Link,
    Title,
    Description,
    Author,
    Tag,
}

impl From<EntryTextField> for colette_crud::EntryTextField {
    fn from(value: EntryTextField) -> Self {
        match value {
            EntryTextField::Title => Self::Title,
            EntryTextField::Link => Self::Link,
            EntryTextField::Description => Self::Description,
            EntryTextField::Author => Self::Author,
            EntryTextField::Tag => Self::Tag,
        }
    }
}

impl From<colette_crud::EntryTextField> for EntryTextField {
    fn from(value: colette_crud::EntryTextField) -> Self {
        match value {
            colette_crud::EntryTextField::Title => Self::Title,
            colette_crud::EntryTextField::Link => Self::Link,
            colette_crud::EntryTextField::Description => Self::Description,
            colette_crud::EntryTextField::Author => Self::Author,
            colette_crud::EntryTextField::Tag => Self::Tag,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EntryBooleanField {
    HasRead,
}

impl From<EntryBooleanField> for colette_crud::EntryBooleanField {
    fn from(value: EntryBooleanField) -> Self {
        match value {
            EntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

impl From<colette_crud::EntryBooleanField> for EntryBooleanField {
    fn from(value: colette_crud::EntryBooleanField) -> Self {
        match value {
            colette_crud::EntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EntryDateField {
    PublishedAt,
}

impl From<EntryDateField> for colette_crud::EntryDateField {
    fn from(value: EntryDateField) -> Self {
        match value {
            EntryDateField::PublishedAt => Self::PublishedAt,
        }
    }
}

impl From<colette_crud::EntryDateField> for EntryDateField {
    fn from(value: colette_crud::EntryDateField) -> Self {
        match value {
            colette_crud::EntryDateField::PublishedAt => Self::PublishedAt,
        }
    }
}
