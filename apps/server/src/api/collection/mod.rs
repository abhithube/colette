use chrono::{DateTime, Utc};
use colette_core::collection;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    ApiState,
    common::{DateOp, TextOp},
};
use crate::api::common::Paginated;

mod create_collection;
mod delete_collection;
mod get_collection;
mod list_collection_bookmarks;
mod list_collections;
mod update_collection;

pub const COLLECTIONS_TAG: &str = "Collections";

#[derive(OpenApi)]
#[openapi(components(schemas(Collection, BookmarkFilter, BookmarkTextField, BookmarkDateField, Paginated<Collection>, create_collection::CollectionCreate, update_collection::CollectionUpdate)))]
pub struct CollectionApi;

impl CollectionApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(CollectionApi::openapi())
            .routes(routes!(
                list_collections::handler,
                create_collection::handler
            ))
            .routes(routes!(
                get_collection::handler,
                update_collection::handler,
                delete_collection::handler
            ))
            .routes(routes!(list_collection_bookmarks::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub filter: BookmarkFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<colette_core::Collection> for Collection {
    fn from(value: colette_core::Collection) -> Self {
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
pub enum BookmarkFilter {
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

impl From<BookmarkFilter> for collection::BookmarkFilter {
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

impl From<collection::BookmarkFilter> for BookmarkFilter {
    fn from(value: collection::BookmarkFilter) -> Self {
        match value {
            collection::BookmarkFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            collection::BookmarkFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            collection::BookmarkFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            collection::BookmarkFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            collection::BookmarkFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkTextField {
    Link,
    Title,
    Author,
    Tag,
}

impl From<BookmarkTextField> for collection::BookmarkTextField {
    fn from(value: BookmarkTextField) -> Self {
        match value {
            BookmarkTextField::Title => Self::Title,
            BookmarkTextField::Link => Self::Link,
            BookmarkTextField::Author => Self::Author,
            BookmarkTextField::Tag => Self::Tag,
        }
    }
}

impl From<collection::BookmarkTextField> for BookmarkTextField {
    fn from(value: collection::BookmarkTextField) -> Self {
        match value {
            collection::BookmarkTextField::Title => Self::Title,
            collection::BookmarkTextField::Link => Self::Link,
            collection::BookmarkTextField::Author => Self::Author,
            collection::BookmarkTextField::Tag => Self::Tag,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkDateField {
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

impl From<BookmarkDateField> for collection::BookmarkDateField {
    fn from(value: BookmarkDateField) -> Self {
        match value {
            BookmarkDateField::PublishedAt => Self::PublishedAt,
            BookmarkDateField::CreatedAt => Self::CreatedAt,
            BookmarkDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}

impl From<collection::BookmarkDateField> for BookmarkDateField {
    fn from(value: collection::BookmarkDateField) -> Self {
        match value {
            collection::BookmarkDateField::PublishedAt => Self::PublishedAt,
            collection::BookmarkDateField::CreatedAt => Self::CreatedAt,
            collection::BookmarkDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}
