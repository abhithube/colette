use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::bookmark::{self, BookmarkService};
use url::Url;

use crate::common::{BOOKMARKS_TAG, BaseError, Error};

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkScrape {
    #[schema(format = "uri")]
    pub url: Url,
}

impl From<BookmarkScrape> for bookmark::BookmarkScrape {
    fn from(value: BookmarkScrape) -> Self {
        Self { url: value.url }
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkScraped {
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri", required)]
    pub thumbnail_url: Option<String>,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub author: Option<String>,
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

#[utoipa::path(
    post,
    path = "/scrape",
    request_body = BookmarkScrape,
    responses(ScrapeResponse),
    operation_id = "scrapeBookmark",
    description = "Scrape bookmark from a webpage",
    tag = BOOKMARKS_TAG
  )]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<BookmarkService>>,
    Json(body): Json<BookmarkScrape>,
) -> Result<ScrapeResponse, Error> {
    match service.scrape_bookmark(body.into()).await {
        Ok(data) => Ok(ScrapeResponse::Ok(data.into())),
        Err(bookmark::Error::Scraper(e)) => Ok(ScrapeResponse::BadGateway(BaseError {
            message: e.to_string(),
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum ScrapeResponse {
    #[response(status = 201, description = "Scraped bookmark")]
    Ok(BookmarkScraped),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse bookmark")]
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
