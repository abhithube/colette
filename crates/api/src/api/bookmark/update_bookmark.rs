use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_crud::BookmarkError;
use colette_handler::{Handler as _, UpdateBookmarkCommand, UpdateBookmarkError};
use url::Url;

use crate::api::{
    ApiState,
    bookmark::BOOKMARKS_TAG,
    common::{ApiError, Auth, Id, Json, NonEmptyString, Path},
};

#[utoipa::path(
  patch,
  path = "/{id}",
  params(Id),
  request_body = BookmarkUpdate,
  responses(OkResponse, ErrResponse),
  operation_id = "updateBookmark",
  description = "Update a bookmark by ID",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<BookmarkUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .update_bookmark
        .handle(UpdateBookmarkCommand {
            id: id.into(),
            title: body.title.map(Into::into),
            thumbnail_url: body.thumbnail_url,
            published_at: body.published_at,
            author: body.author.map(|e| e.map(Into::into)),
            user_id,
        })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => match e {
            UpdateBookmarkError::Bookmark(BookmarkError::NotFound(_)) => {
                Err(ErrResponse::NotFound(e.into()))
            }
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Updates to make to an existing bookmark
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkUpdate {
    /// Human-readable name for the bookmark to update, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    title: Option<NonEmptyString>,
    /// Thumbnail URL of the bookmark to update, will be archived
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<Url>)]
    thumbnail_url: Option<Option<Url>>,
    /// Timestamp at which the bookmark was published
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<DateTime<Utc>>)]
    published_at: Option<Option<DateTime<Utc>>>,
    /// Author of the bookmark to update, cannot be empty
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<Option<String>>, min_length = 1)]
    author: Option<Option<NonEmptyString>>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully updated bookmark")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Bookmark not found")]
    NotFound(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
