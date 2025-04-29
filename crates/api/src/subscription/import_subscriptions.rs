use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;

use super::SUBSCRIPTIONS_TAG;
use crate::{
    ApiState,
    common::{ApiError, AuthUser},
};

#[utoipa::path(
  post,
  path = "/import",
  request_body = Vec<u8>,
  responses(OkResponse, ErrResponse),
  operation_id = "importSubscriptions",
  description = "Import subscriptions into user account",
  tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    bytes: Bytes,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .import_subscriptions(bytes, user_id)
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully started import")]
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

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
