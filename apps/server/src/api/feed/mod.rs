use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, common::Paginated, tag::Tag};

mod create_feed;
mod delete_feed;
mod detect_feeds;
mod get_feed;
mod list_feeds;
mod update_feed;

pub const FEEDS_TAG: &str = "Feeds";

#[derive(OpenApi)]
#[openapi(components(schemas(
    Feed,
    Paginated<Feed>,
    create_feed::FeedCreate,
    update_feed::FeedUpdate,
    detect_feeds::FeedDetect,
    detect_feeds::FeedDetected,
    detect_feeds::FeedProcessed,
    detect_feeds::DetectedResponse
)))]
pub struct FeedApi;

impl FeedApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(FeedApi::openapi())
            .routes(routes!(list_feeds::handler, create_feed::handler))
            .routes(routes!(
                get_feed::handler,
                update_feed::handler,
                delete_feed::handler
            ))
            .routes(routes!(detect_feeds::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    #[schema(required)]
    pub xml_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[schema(nullable = false)]
    pub unread_count: Option<i64>,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            xml_url: value.xml_url,
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
            unread_count: value.unread_count,
        }
    }
}
