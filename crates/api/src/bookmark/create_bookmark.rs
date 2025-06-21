use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::bookmark;
use url::Url;

use super::{BOOKMARKS_TAG, Bookmark};
use crate::{
    ApiState,
    common::{ApiError, Auth, Json, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = BookmarkCreate,
  responses(OkResponse, ErrResponse),
  operation_id = "createBookmark",
  description = "Add a bookmark",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    Json(body): Json<BookmarkCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .bookmark_service
        .create_bookmark(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            bookmark::Error::Conflict(_) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Data to create a new bookmark
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkCreate {
    /// URL of the webpage the bookmark links to
    url: Url,
    /// Human-readable name for the new bookmark, cannot be empty
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
    /// Thumbnail URL of the new bookmark, will be archived
    thumbnail_url: Option<Url>,
    /// Timestamp at which the bookmark was published
    published_at: Option<DateTime<Utc>>,
    /// Author for the new bookmark, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1)]
    author: Option<NonEmptyString>,
}

impl From<BookmarkCreate> for bookmark::BookmarkCreate {
    fn from(value: BookmarkCreate) -> Self {
        Self {
            url: value.url,
            title: value.title.into(),
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author.map(Into::into),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Created bookmark")]
pub(super) struct OkResponse(Bookmark);

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

    #[response(status = StatusCode::CONFLICT, description = "Bookmark already exists")]
    Conflict(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
