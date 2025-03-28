use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};

use super::{COLLECTIONS_TAG, Collection};
use crate::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listCollections",
    description = "List user collections",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state.collection_service.list_collections(user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of collections")]
    Ok(Paginated<Collection>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
