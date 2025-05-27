use axum::{Router, routing};
use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use super::{ApiState, common::Paginated};

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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeedEntry {
    id: Uuid,
    link: Url,
    title: String,
    published_at: DateTime<Utc>,
    #[schema(required)]
    description: Option<String>,
    #[schema(required)]
    author: Option<String>,
    #[schema(required)]
    thumbnail_url: Option<Url>,
    feed_id: Uuid,
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
            feed_id: value.feed_id,
        }
    }
}
