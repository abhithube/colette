use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::bookmark;
use url::Url;

use super::BOOKMARKS_TAG;
use crate::{
    ApiState,
    common::{ApiError, Json},
};

#[utoipa::path(
    post,
    path = "/scrape",
    request_body = BookmarkScrape,
    responses(OkResponse, ErrResponse),
    operation_id = "scrapeBookmark",
    description = "Scrape bookmark from a webpage",
    tag = BOOKMARKS_TAG
  )]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<BookmarkScrape>,
) -> Result<OkResponse, ErrResponse> {
    match state.bookmark_service.scrape_bookmark(body.into()).await {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(bookmark::Error::Scraper(e)) => Err(ErrResponse::BadGateway(e.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkScrape {
    url: Url,
}

impl From<BookmarkScrape> for bookmark::BookmarkScrape {
    fn from(value: BookmarkScrape) -> Self {
        Self { url: value.url }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkScraped {
    link: Url,
    title: String,
    #[schema(required)]
    thumbnail_url: Option<Url>,
    #[schema(required)]
    published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    author: Option<String>,
}

impl From<bookmark::BookmarkScraped> for BookmarkScraped {
    fn from(value: bookmark::BookmarkScraped) -> Self {
        Self {
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Scraped bookmark")]
pub(super) struct OkResponse(BookmarkScraped);

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
