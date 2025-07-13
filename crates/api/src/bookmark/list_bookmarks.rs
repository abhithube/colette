use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::bookmark::{self, BookmarkCursor};
use uuid::Uuid;

use super::{BOOKMARKS_TAG, BookmarkDetails};
use crate::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
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
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .bookmark_service
        .list_bookmarks(query.try_into()?, user_id)
        .await
    {
        Ok(bookmarks) => {
            let data = bookmarks
                .try_into()
                .map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct BookmarkListQuery {
    /// Filter by the ID of a collection whose filters may apply to the bookmark
    #[param(nullable = false)]
    collection_id: Option<Uuid>,
    /// Whether to filter by tags linked to the bookmark
    #[param(nullable = false)]
    filter_by_tags: Option<bool>,
    /// Filter by the IDs of the tags linked to the bookmark
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    tags: Option<Vec<Uuid>>,
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
    /// Whether to include the tags linked to the bookmark
    #[serde(default = "with_tags")]
    with_tags: bool,
}

fn with_tags() -> bool {
    false
}

impl TryFrom<BookmarkListQuery> for bookmark::BookmarkListQuery {
    type Error = ErrResponse;

    fn try_from(value: BookmarkListQuery) -> Result<Self, Self::Error> {
        let cursor = value
            .cursor
            .map(|e| decode_cursor::<BookmarkCursor>(&e))
            .transpose()
            .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

        Ok(Self {
            collection_id: value.collection_id,
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
            cursor,
            limit: Some(PAGINATION_LIMIT),
            with_tags: value.with_tags,
        })
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
