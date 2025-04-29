use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed;
use url::Url;

use super::FEEDS_TAG;
use crate::{
    ApiState,
    common::{ApiError, Json},
};

#[utoipa::path(
    post,
    path = "/detect",
    request_body = FeedDetect,
    responses(OkResponse, ErrResponse),
    operation_id = "detectFeeds",
    description = "Detects web feeds on a page",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<FeedDetect>,
) -> Result<OkResponse, ErrResponse> {
    match state.feed_service.detect_feeds(body.into()).await {
        Ok(data) => Ok(OkResponse(data.into_iter().map(Into::into).collect())),
        Err(feed::Error::Scraper(e)) => Err(ErrResponse::BadGateway(e.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct FeedDetect {
    url: Url,
}

impl From<FeedDetect> for feed::FeedDetect {
    fn from(value: FeedDetect) -> Self {
        Self { url: value.url }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct FeedDetected {
    url: Url,
    title: String,
}

impl From<feed::FeedDetected> for FeedDetected {
    fn from(value: feed::FeedDetected) -> Self {
        Self {
            url: value.url,
            title: value.title,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "List of detected feeds")]
pub(super) struct OkResponse(Vec<FeedDetected>);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = StatusCode::BAD_GATEWAY, description = "Failed to fetch data")]
    BadGateway(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
