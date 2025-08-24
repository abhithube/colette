use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::entry;
use colette_handler::EntryDto;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    ApiState,
    common::{BooleanOp, DateOp, TextOp},
    pagination::Paginated,
};

mod list_entries;
mod mark_entry_as_read;
mod mark_entry_as_unread;

const ENTRIES_TAG: &str = "Entries";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Entry, Paginated<Entry>, EntryFilter, EntryTextField, EntryBooleanField, EntryDateField)),
    paths(list_entries::handler, mark_entry_as_read::handler, mark_entry_as_unread::handler)
)]
pub(crate) struct EntryApi;

impl EntryApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_entries::handler))
            .route(
                "/{id}/markAsRead",
                routing::post(mark_entry_as_read::handler),
            )
            .route(
                "/{id}/markAsUnread",
                routing::post(mark_entry_as_unread::handler),
            )
    }
}

/// A feed entry, with read status
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Entry {
    /// Unique identifier of the entry
    id: Uuid,
    /// URL of the webpage the entry links to
    link: Url,
    /// Title of the entry
    title: String,
    /// Timestamp at which the entry was published
    published_at: DateTime<Utc>,
    /// Description of the entry
    #[schema(required)]
    description: Option<String>,
    /// Author of the entry
    #[schema(required)]
    author: Option<String>,
    /// Thumbnail URL of the entry
    #[schema(required)]
    thumbnail_url: Option<Url>,
    /// Read status of the entry
    read_status: ReadStatus,
    /// Unique identifier of the associated feed
    feed_id: Uuid,
}

impl From<EntryDto> for Entry {
    fn from(value: EntryDto) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            read_status: value.read_status.into(),
            feed_id: value.feed_id,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub enum ReadStatus {
    Unread,
    Read(DateTime<Utc>),
}

impl From<entry::ReadStatus> for ReadStatus {
    fn from(value: entry::ReadStatus) -> Self {
        match value {
            entry::ReadStatus::Unread => ReadStatus::Unread,
            entry::ReadStatus::Read(read_at) => ReadStatus::Read(read_at),
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

impl From<EntryFilter> for entry::EntryFilter {
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

impl From<entry::EntryFilter> for EntryFilter {
    fn from(value: entry::EntryFilter) -> Self {
        match value {
            entry::EntryFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            entry::EntryFilter::Boolean { field, op } => Self::Boolean {
                field: field.into(),
                op: op.into(),
            },
            entry::EntryFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            entry::EntryFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            entry::EntryFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            entry::EntryFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
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

impl From<EntryTextField> for entry::EntryTextField {
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

impl From<entry::EntryTextField> for EntryTextField {
    fn from(value: entry::EntryTextField) -> Self {
        match value {
            entry::EntryTextField::Title => Self::Title,
            entry::EntryTextField::Link => Self::Link,
            entry::EntryTextField::Description => Self::Description,
            entry::EntryTextField::Author => Self::Author,
            entry::EntryTextField::Tag => Self::Tag,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EntryBooleanField {
    HasRead,
}

impl From<EntryBooleanField> for entry::EntryBooleanField {
    fn from(value: EntryBooleanField) -> Self {
        match value {
            EntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

impl From<entry::EntryBooleanField> for EntryBooleanField {
    fn from(value: entry::EntryBooleanField) -> Self {
        match value {
            entry::EntryBooleanField::HasRead => Self::HasRead,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EntryDateField {
    PublishedAt,
}

impl From<EntryDateField> for entry::EntryDateField {
    fn from(value: EntryDateField) -> Self {
        match value {
            EntryDateField::PublishedAt => Self::PublishedAt,
        }
    }
}

impl From<entry::EntryDateField> for EntryDateField {
    fn from(value: entry::EntryDateField) -> Self {
        match value {
            entry::EntryDateField::PublishedAt => Self::PublishedAt,
        }
    }
}
