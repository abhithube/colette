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
    common::{ApiError, AuthUser, Id, Json, NonEmptyString, Path},
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
    AuthUser(user): AuthUser,
    Json(body): Json<BookmarkUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .bookmark_service
        .update_bookmark(id, body.into(), user.id)
        .await
    {
        Ok(data) => Ok(OkResponse(
            (data, state.config.storage.base_url.clone()).into(),
        )),
        Err(e) => match e {
            bookmark::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            bookmark::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct BookmarkUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    title: Option<NonEmptyString>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<Url>)]
    thumbnail_url: Option<Option<Url>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<DateTime<Utc>>)]
    published_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<Option<String>>, min_length = 1)]
    author: Option<Option<NonEmptyString>>,
}

impl From<BookmarkUpdate> for bookmark::BookmarkUpdate {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author.map(|e| e.map(Into::into)),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Updated bookmark")]
pub(super) struct OkResponse(Bookmark);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

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
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
