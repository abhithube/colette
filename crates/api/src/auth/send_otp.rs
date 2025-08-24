use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_handler::{Handler as _, SendOtpCommand};

use crate::{
    ApiState,
    auth::AUTH_TAG,
    common::{ApiError, Json},
};

#[utoipa::path(
  post,
  path = "/send-otp",
  request_body = SendOtpPayload,
  responses(OkResponse, ErrResponse),
  operation_id = "sendOtp",
  description = "Send an OTP code to an email",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<SendOtpPayload>,
) -> Result<impl IntoResponse, ErrResponse> {
    match state
        .send_otp
        .handle(SendOtpCommand { email: body.email })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SendOtpPayload {
    #[schema(format = "email")]
    email: String,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully sent OTP code")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

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
