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
    path = "/scrape",
    request_body = FeedScrape,
    responses(ScrapeResponse),
    operation_id = "scrapeFeed",
    description = "Scrape web feed",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<FeedScrape>,
) -> Result<ScrapeResponse, Error> {
    match state.feed_service.refresh_feed(body.into()).await {
        Ok(data) => Ok(ScrapeResponse::Ok(data.into())),
        Err(feed::Error::Scraper(e)) => Ok(ScrapeResponse::BadGateway(BaseError {
            message: e.to_string(),
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedScrape {
    pub url: Url,
}

impl From<FeedScrape> for feed::FeedRefresh {
    fn from(value: FeedScrape) -> Self {
        Self { url: value.url }
    }
}

#[allow(dead_code)]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum ScrapeResponse {
    #[response(status = 200, description = "Scraped feed")]
    Ok(Feed),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse feed")]
    BadGateway(BaseError),
}

impl IntoResponse for ScrapeResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
    }
}
