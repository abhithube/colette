use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};

use super::{LIBRARY_TAG, LibraryItem};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
    library::LibraryItemListQuery,
};

#[utoipa::path(
    get,
    path = "",
    params(LibraryItemListQuery),
    responses(ListResponse),
    operation_id = "listLibraryItems",
    description = "List user library items, consisting of folders, feeds, and collections",
    tag = LIBRARY_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<LibraryItemListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, Error> {
    match state
        .library_service
        .list_library_items(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(Paginated {
            data: data.data.into_iter().map(Into::into).collect(),
            cursor: data.cursor,
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(
        status = 200,
        description = "Paginated list of folders, feeds, and collections"
    )]
    Ok(Paginated<LibraryItem>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
