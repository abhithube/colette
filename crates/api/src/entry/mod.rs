use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_handler::EntryDto;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated};

mod list_entries;
mod mark_entry_as_read;
mod mark_entry_as_unread;

const ENTRIES_TAG: &str = "Entries";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Entry, Paginated<Entry>)),
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

impl From<colette_crud::ReadStatus> for ReadStatus {
    fn from(value: colette_crud::ReadStatus) -> Self {
        match value {
            colette_crud::ReadStatus::Unread => ReadStatus::Unread,
            colette_crud::ReadStatus::Read(read_at) => ReadStatus::Read(read_at),
        }
    }
}
