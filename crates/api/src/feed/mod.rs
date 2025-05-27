use axum::{Router, routing};
use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use super::ApiState;

mod detect_feeds;
mod scrape_feed;

const FEEDS_TAG: &str = "Feeds";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Feed, detect_feeds::FeedDetect, detect_feeds::FeedDetected)),
    paths(detect_feeds::handler, scrape_feed::handler)
)]
pub(crate) struct FeedApi;

impl FeedApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/detect", routing::post(detect_feeds::handler))
            .route("/scrape", routing::post(scrape_feed::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Feed {
    id: Uuid,
    source_url: Url,
    link: Url,
    title: String,
    #[schema(required)]
    description: Option<String>,
    #[schema(required)]
    refreshed_at: Option<DateTime<Utc>>,
    is_custom: bool,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            source_url: value.source_url,
            link: value.link,
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at,
            is_custom: value.is_custom,
        }
    }
}
