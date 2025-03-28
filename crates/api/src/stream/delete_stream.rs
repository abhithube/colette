use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::stream;

use super::STREAMS_TAG;
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteStream",
    description = "Delete a stream by ID",
    tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
) -> Result<DeleteResponse, Error> {
    match state.stream_service.delete_stream(id, user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            stream::Error::Forbidden(_) => Ok(DeleteResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            stream::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted stream")]
    NoContent,

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Stream not found")]
    NotFound(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
