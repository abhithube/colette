use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use colette_core::library;
use uuid::Uuid;

use super::{CollectionTreeItem, LIBRARY_TAG};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "/collectionTree",
    params(LibraryItemListQuery),
    responses(ListResponse),
    operation_id = "listCollectionTree",
    description = "List user collection tree, consisting of folders and collections",
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
        .list_collection_tree(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(Paginated {
            data: data.data.into_iter().map(Into::into).collect(),
            cursor: data.cursor,
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(
        status = 200,
        description = "Paginated list of folders and collections"
    )]
    Ok(Paginated<CollectionTreeItem>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
