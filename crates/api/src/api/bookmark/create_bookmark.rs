use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_crud::BookmarkError;
use colette_handler::{CreateBookmarkCommand, CreateBookmarkError, Handler as _};
use url::Url;

use crate::api::{
    ApiState,
    bookmark::BOOKMARKS_TAG,
    common::{ApiError, Auth, CreatedResource, Json, NonEmptyString},
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
        .create_bookmark
        .handle(CreateBookmarkCommand {
            url: body.url,
            title: body.title.into(),
            thumbnail_url: body.thumbnail_url,
            published_at: body.published_at,
            author: body.author.map(Into::into),
            user_id,
        })
        .await
    {
        Ok(data) => Ok(OkResponse(CreatedResource {
            id: data.id().as_inner(),
        })),
        Err(e) => match e {
            CreateBookmarkError::Bookmark(BookmarkError::Conflict(_)) => {
                Err(ErrResponse::Conflict(e.into()))
            }
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

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "New bookmark ID")]
pub(super) struct OkResponse(CreatedResource);

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
