use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};

use super::{CollectionTreeItem, LIBRARY_TAG};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
    library::TreeListQuery,
};

#[utoipa::path(
    get,
    path = "/collectionTree",
    params(TreeListQuery),
    responses(ListResponse),
    operation_id = "listCollectionTree",
    description = "List user collection tree, consisting of folders and collections",
    tag = LIBRARY_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<TreeListQuery>,
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
