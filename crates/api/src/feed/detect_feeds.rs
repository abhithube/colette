use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed;
use url::Url;

use super::{FEEDS_TAG, Feed};
use crate::{
    ApiState,
    common::{BaseError, Error},
};

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
    State(state): State<ApiState>,
    Json(body): Json<FeedDetect>,
) -> Result<DetectResponse, Error> {
    match state.feed_service.detect_feeds(body.into()).await {
        Ok(data) => Ok(DetectResponse::Ok(data.into())),
        Err(feed::Error::Scraper(e)) => Ok(DetectResponse::BadGateway(BaseError {
            message: e.to_string(),
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetect {
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
    pub url: Url,
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum DetectedResponse {
    Detected(Vec<FeedDetected>),
    Processed(Feed),
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

#[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
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
