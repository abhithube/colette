use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed;
use url::Url;

use super::{FEEDS_TAG, Feed};
use crate::{
    ApiState,
    common::{ApiError, Json},
};

#[utoipa::path(
    post,
    path = "/scrape",
    request_body = FeedScrape,
    responses(OkResponse, ErrResponse),
    operation_id = "scrapeFeed",
    description = "Scrape web feed",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<FeedScrape>,
) -> Result<OkResponse, ErrResponse> {
    match state.feed_service.refresh_feed(body.into()).await {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(feed::Error::Scraper(e)) => Err(ErrResponse::BadGateway(e.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct FeedScrape {
    url: Url,
}

impl From<FeedScrape> for feed::FeedRefresh {
    fn from(value: FeedScrape) -> Self {
        Self { url: value.url }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Scraped feed")]
pub(super) struct OkResponse(Feed);

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
