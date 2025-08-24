use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_handler::{Handler as _, ScrapeBookmarkCommand};
use url::Url;

use crate::{
    ApiState,
    bookmark::BOOKMARKS_TAG,
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
    match state
        .scrape_bookmark
        .handle(ScrapeBookmarkCommand { url: body.url })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::BadGateway(e.into())),
    }
}

/// Data to scrape a bookmark using
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkScrape {
    /// URL of a webpage to scrape
    url: Url,
}

/// Scraped bookmark
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkScraped {
    /// URL of the webpage of the scraped bookmark
    link: Url,
    /// Title of the scraped bookmark
    title: String,
    /// Thumbnail URL of the scraped bookmark
    #[schema(required)]
    thumbnail_url: Option<Url>,
    /// Timestamp at which the scraped bookmark was published
    #[schema(required)]
    published_at: Option<DateTime<Utc>>,
    /// Author of the scraped bookmark
    #[schema(required)]
    author: Option<String>,
}

impl From<colette_handler::BookmarkScraped> for BookmarkScraped {
    fn from(value: colette_handler::BookmarkScraped) -> Self {
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
