use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::collection;

use super::{COLLECTIONS_TAG, Collection};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getCollection",
    description = "Get a collection by ID",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
) -> Result<GetResponse, Error> {
    match state.collection_service.get_collection(id, user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            collection::Error::Forbidden(_) => Ok(GetResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            collection::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Collection by ID")]
    Ok(Collection),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Collection not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
