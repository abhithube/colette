use axum::{Router, routing};
use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated};

mod get_feed_entry;
mod list_feed_entries;

const FEED_ENTRIES_TAG: &str = "Feed Entries";

#[derive(OpenApi)]
#[openapi(
    components(schemas(FeedEntry, Paginated<FeedEntry>)),
    paths(list_feed_entries::handler, get_feed_entry::handler)
)]
pub(crate) struct FeedEntryApi;

impl FeedEntryApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_feed_entries::handler))
            .route("/{id}", routing::get(get_feed_entry::handler))
    }
}

/// RSS feed entry
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeedEntry {
    /// Unique identifier of the feed entry
    id: Uuid,
    /// URL of the webpage the feed entry links to
    link: Url,
    /// Title of the feed entry
    title: String,
    /// Timestamp at which the feed entry was published
    published_at: DateTime<Utc>,
    /// Description of the feed entry
    #[schema(required)]
    description: Option<String>,
    /// Author of the feed entry
    #[schema(required)]
    author: Option<String>,
    /// Thumbnail URL of the feed entry
    #[schema(required)]
    thumbnail_url: Option<Url>,
    /// Unique identifier of the associated RSS feed
    feed_id: Uuid,
}

impl From<colette_core::FeedEntry> for FeedEntry {
    fn from(value: colette_core::FeedEntry) -> Self {
        Self {
            id: value.id.as_inner(),
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            feed_id: value.feed_id.as_inner(),
        }
    }
}
