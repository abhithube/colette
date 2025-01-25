use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use colette_core::library::{self, LibraryService};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    bookmark::Bookmark,
    common::{Error, Session, LIBRARY_TAG},
    feed::Feed,
    folder::Folder,
    Paginated,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct LibraryState {
    service: Arc<LibraryService>,
}

impl LibraryState {
    pub fn new(service: Arc<LibraryService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(LibraryItem, Paginated<LibraryItem>)))]
pub struct LibraryApi;

impl LibraryApi {
    pub fn router() -> OpenApiRouter<LibraryState> {
        OpenApiRouter::with_openapi(LibraryApi::openapi()).routes(routes!(list_library_items))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum LibraryItem {
    Folder(Folder),
    Feed(Feed),
    Bookmark(Bookmark),
}

impl From<colette_core::LibraryItem> for LibraryItem {
    fn from(value: colette_core::LibraryItem) -> Self {
        match value {
            colette_core::LibraryItem::Folder(folder) => Self::Folder(folder.into()),
            colette_core::LibraryItem::Feed(feed) => Self::Feed(feed.into()),
            colette_core::LibraryItem::Bookmark(bookmark) => Self::Bookmark(bookmark.into()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct LibraryItemListQuery {
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<LibraryItemListQuery> for library::LibraryItemListQuery {
    fn from(value: LibraryItemListQuery) -> Self {
        Self {
            folder_id: value.folder_id,
            cursor: value.cursor,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(LibraryItemListQuery),
    responses(ListResponse),
    operation_id = "listLibraryItems",
    description = "List user library items, consisting of folders, feeds, and bookmarks",
    tag = LIBRARY_TAG
)]
#[axum::debug_handler]
pub async fn list_library_items(
    State(service): State<Arc<LibraryService>>,
    Query(query): Query<LibraryItemListQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service
        .list_library_items(query.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}
#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of folders")]
    Ok(Paginated<LibraryItem>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
