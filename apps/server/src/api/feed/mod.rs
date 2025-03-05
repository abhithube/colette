use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ApiState;

mod detect_feeds;

pub const FEEDS_TAG: &str = "Feeds";

#[derive(OpenApi)]
#[openapi(components(schemas(
    detect_feeds::FeedDetect,
    detect_feeds::FeedDetected,
    detect_feeds::DetectedResponse
)))]
pub struct FeedApi;

impl FeedApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(FeedApi::openapi()).routes(routes!(detect_feeds::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: Uuid,
    pub link: Url,
    #[schema(required)]
    pub xml_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            xml_url: value.xml_url,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
