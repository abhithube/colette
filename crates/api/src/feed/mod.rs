use axum::{Router, routing};
use url::Url;
use utoipa::OpenApi;

use crate::ApiState;

mod detect_feeds;
mod scrape_feed;

const FEEDS_TAG: &str = "Feeds";

#[derive(OpenApi)]
#[openapi(
    components(schemas(FeedScraped, detect_feeds::FeedDetect, detect_feeds::FeedDetected)),
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

/// RSS feed
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeedScraped {
    /// URL to scrape for feed updates
    source_url: Url,
    /// URL of the webpage the feed links to
    link: Url,
    /// Title of the feed
    title: String,
    /// Description of the feed
    #[schema(required)]
    description: Option<String>,
}

impl From<colette_handler::FeedCreated> for FeedScraped {
    fn from(value: colette_handler::FeedCreated) -> Self {
        Self {
            source_url: value.source_url,
            link: value.link,
            title: value.title,
            description: value.description,
        }
    }
}
