use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::pat::PatError;
use colette_handler::{DeletePatCommand, DeletePatError, Handler as _};

use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
    pat::PERSONAL_ACCESS_TOKENS_TAG,
};

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(OkResponse, ErrResponse),
    operation_id = "deletePat",
    description = "Delete a PAT by ID",
    tag = PERSONAL_ACCESS_TOKENS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .delete_pat
        .handle(DeletePatCommand {
            id: id.into(),
            user_id,
        })
        .await
    {
        Ok(()) => Ok(OkResponse),
        Err(e) => match e {
            DeletePatError::Pat(e) => match e {
                PatError::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
                _ => Err(ErrResponse::InternalServerError(e.into())),
            },
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully deleted API key")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "API key not found")]
    NotFound(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
