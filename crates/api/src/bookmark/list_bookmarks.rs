use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::bookmark;
use uuid::Uuid;

use super::{BOOKMARKS_TAG, BookmarkDetails};
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Paginated, Query},
};

#[utoipa::path(
  get,
  path = "",
  params(BookmarkListQuery),
  responses(OkResponse, ErrResponse),
  operation_id = "listBookmarks",
  description = "List user bookmarks",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<BookmarkListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<OkResponse, ErrResponse> {
    match state
        .bookmark_service
        .list_bookmarks(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(OkResponse(Paginated {
            data: data
                .data
                .into_iter()
                .map(|e| (e, state.image_base_url.clone()).into())
                .collect(),
            cursor: data.cursor,
        })),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct BookmarkListQuery {
    #[param(nullable = false)]
    collection_id: Option<Uuid>,
    #[param(nullable = false)]
    filter_by_tags: Option<bool>,
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    tags: Option<Vec<Uuid>>,
    #[param(nullable = false)]
    cursor: Option<String>,
    #[serde(default = "with_tags")]
    with_tags: bool,
}

fn with_tags() -> bool {
    false
}

impl From<BookmarkListQuery> for bookmark::BookmarkListQuery {
    fn from(value: BookmarkListQuery) -> Self {
        Self {
            collection_id: value.collection_id,
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
            cursor: value.cursor,
            with_tags: value.with_tags,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of bookmarks")]
pub(super) struct OkResponse(Paginated<BookmarkDetails>);

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

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
