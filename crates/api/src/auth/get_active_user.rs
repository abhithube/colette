use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::{AUTH_TAG, User};
use crate::common::{ApiError, AuthUser};

#[utoipa::path(
  get,
  path = "/@me",
  responses(OkResponse, ErrResponse),
  operation_id = "getActiveUser",
  description = "Get the active user",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(AuthUser(user): AuthUser) -> Result<OkResponse, ErrResponse> {
    Ok(OkResponse(user.into()))
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
