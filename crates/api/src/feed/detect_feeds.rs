use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    feed::{self, DetectFeedsCommand, DetectFeedsError},
};
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
    match state
        .detect_feeds
        .handle(DetectFeedsCommand { url: body.url })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into_iter().map(Into::into).collect())),
        Err(DetectFeedsError::Scraper(e)) => Err(ErrResponse::BadGateway(e.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

/// Data to detect RSS feeds using
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct FeedDetect {
    /// URL of a webpage to detect RSS feeds on
    url: Url,
}

/// Detected RSS feed
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct FeedDetected {
    /// URL of the detected RSS feed
    url: Url,
    /// Title of the detected RSS feed
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
