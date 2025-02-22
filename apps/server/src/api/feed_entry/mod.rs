use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, common::Paginated};

mod get_feed_entry;
mod list_feed_entries;
mod update_feed_entry;

pub const FEED_ENTRIES_TAG: &str = "Feed Entries";

#[derive(OpenApi)]
#[openapi(components(schemas(FeedEntry, Paginated<FeedEntry>, update_feed_entry::FeedEntryUpdate)))]
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
