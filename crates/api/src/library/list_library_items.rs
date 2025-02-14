use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use colette_core::library::{self, LibraryService};
use uuid::Uuid;

use super::LibraryItem;
use crate::{
    Paginated,
    common::{Error, LIBRARY_TAG, Session},
};

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
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
pub async fn handler(
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
