use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use super::COLLECTIONS_TAG;
use crate::api::{
    ApiState,
    bookmark::Bookmark,
    common::{AuthUser, Error, Id, Paginated},
};

#[utoipa::path(
    get,
    path = "/{id}/bookmarks",
    params(Id),
    responses(ListResponse),
    operation_id = "listCollectionBookmarks",
    description = "List collection bookmarks",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state
        .collection_service
        .list_collection_bookmarks(id, user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(Paginated {
            data: data
                .data
                .into_iter()
                .map(|e| (e, state.bucket_url.clone()).into())
                .collect(),
            cursor: data.cursor,
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of collection bookmarks")]
    Ok(Paginated<Bookmark>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
