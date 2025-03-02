use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use colette_core::collection;

use super::COLLECTIONS_TAG;
use crate::api::{
    ApiState,
    bookmark::Bookmark,
    common::{AuthUser, Error, Id, Paginated},
};

#[utoipa::path(
    get,
    path = "/{id}/bookmarks",
    params(Id, CollectionBookmarkListQuery),
    responses(ListResponse),
    operation_id = "listCollectionBookmarks",
    description = "List collection bookmarks",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<CollectionBookmarkListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state
        .collection_service
        .list_collection_bookmarks(id, query.into(), user_id)
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
pub struct CollectionBookmarkListQuery {
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<CollectionBookmarkListQuery> for collection::CollectionBookmarkListQuery {
    fn from(value: CollectionBookmarkListQuery) -> Self {
        Self {
            cursor: value.cursor,
        }
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
