use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::bookmark;

use super::{BOOKMARKS_TAG, BookmarkDetails};
use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Path, Query},
};

#[utoipa::path(
  get,
  path = "/{id}",
  params(Id, BookmarkGetQuery),
  responses(OkResponse, ErrResponse),
  operation_id = "getBookmark",
  description = "Get a bookmark by ID",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<BookmarkGetQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .bookmark_service
        .get_bookmark(
            bookmark::BookmarkGetQuery {
                id,
                with_tags: query.with_tags,
            },
            user_id,
        )
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

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct BookmarkGetQuery {
    /// Whether to include the tags linked to the bookmark
    #[serde(default = "with_tags")]
    with_tags: bool,
}

fn with_tags() -> bool {
    false
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Bookmark by ID")]
pub(super) struct OkResponse(BookmarkDetails);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
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
