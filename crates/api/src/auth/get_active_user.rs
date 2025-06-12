use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::{AUTH_TAG, User};
use crate::{
    ApiState,
    common::{ApiError, Auth},
};

#[utoipa::path(
  get,
  path = "/@me",
  responses(OkResponse, ErrResponse),
  operation_id = "getActiveUser",
  description = "Get the active user",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state.auth_service.get_user(user_id).await {
        Ok(user) => Ok(OkResponse(user.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Active user")]
pub(super) struct OkResponse(User);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, ApiError::not_authenticated()).into_response()
            }
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
        }
    }
}
