use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::bookmark;
use uuid::Uuid;

use super::{BOOKMARKS_TAG, Bookmark};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
  get,
  path = "",
  params(BookmarkListQuery),
  responses(ListResponse),
  operation_id = "listBookmarks",
  description = "List user bookmarks",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<BookmarkListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state
        .bookmark_service
        .list_bookmarks(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(Paginated {
            data: data
                .data
                .into_iter()
                .map(|e| (e, state.image_base_url.clone()).into())
                .collect(),
            cursor: data.cursor,
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct BookmarkListQuery {
    #[param(nullable = false)]
    pub collection_id: Option<Uuid>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<Uuid>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
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
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of bookmarks")]
    Ok(Paginated<Bookmark>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
