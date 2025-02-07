use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::bookmark::{self, BookmarkService};
use uuid::Uuid;

use super::Bookmark;
use crate::api::common::{BOOKMARKS_TAG, Error, Paginated, Session};

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct BookmarkListQuery {
    #[param(nullable = false)]
    pub filter_by_folder: Option<bool>,
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<BookmarkListQuery> for bookmark::BookmarkListQuery {
    fn from(value: BookmarkListQuery) -> Self {
        Self {
            folder_id: if value.filter_by_folder.unwrap_or(value.folder_id.is_some()) {
                Some(value.folder_id)
            } else {
                None
            },
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
            cursor: value.cursor,
        }
    }
}

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
    State(service): State<Arc<BookmarkService>>,
    Query(query): Query<BookmarkListQuery>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_bookmarks(query.into(), session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
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
