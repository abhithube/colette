use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed::{self, FeedService};
use url::Url;

use crate::api::common::{BaseError, Error, FEEDS_TAG};

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetect {
    #[schema(format = "uri")]
    pub url: Url,
}

impl From<FeedDetect> for feed::FeedDetect {
    fn from(value: FeedDetect) -> Self {
        Self { url: value.url }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetected {
    #[schema(format = "uri")]
    pub url: String,
    pub title: String,
}

impl From<feed::FeedDetected> for FeedDetected {
    fn from(value: feed::FeedDetected) -> Self {
        Self {
            url: value.url,
            title: value.title,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedProcessed {
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
}

impl From<feed::ProcessedFeed> for FeedProcessed {
    fn from(value: feed::ProcessedFeed) -> Self {
        Self {
            link: value.link.to_string(),
            title: value.title,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum DetectedResponse {
    Detected(Vec<FeedDetected>),
    Processed(FeedProcessed),
}

impl From<feed::DetectedResponse> for DetectedResponse {
    fn from(value: feed::DetectedResponse) -> Self {
        match value {
            feed::DetectedResponse::Detected(feeds) => {
                Self::Detected(feeds.into_iter().map(FeedDetected::from).collect())
            }
            feed::DetectedResponse::Processed(feed) => Self::Processed(feed.into()),
        }
    }
}

#[utoipa::path(
    post,
    path = "/detect",
    request_body = FeedDetect,
    responses(DetectResponse),
    operation_id = "detectFeeds",
    description = "Detects web feeds on a page",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<FeedService>>,
    Json(body): Json<FeedDetect>,
) -> Result<DetectResponse, Error> {
    match service.detect_feeds(body.into()).await {
        Ok(data) => Ok(DetectResponse::Ok(data.into())),
        Err(feed::Error::Scraper(e)) => Ok(DetectResponse::BadGateway(BaseError {
            message: e.to_string(),
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum DetectResponse {
    #[response(status = 200, description = "Detected feeds")]
    Ok(DetectedResponse),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse feed")]
    BadGateway(BaseError),
}

impl IntoResponse for DetectResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
    }
}
