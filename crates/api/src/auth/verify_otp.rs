use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use colette_handler::{Handler as _, LoginUserError, VerifyOtpCommand};

use crate::{
    ApiState,
    auth::{AUTH_TAG, REFRESH_COOKIE, TokenData},
    common::{ApiError, Json, build_cookie},
};

#[utoipa::path(
  post,
  path = "/verify-otp",
  request_body = VerifyOtpPayload,
  responses(OkResponse, ErrResponse),
  operation_id = "verifyOtp",
  description = "Verify an OTP code and log in a user",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    jar: CookieJar,
    Json(body): Json<VerifyOtpPayload>,
) -> Result<impl IntoResponse, ErrResponse> {
    match state
        .verify_otp
        .handle(VerifyOtpCommand {
            email: body.email,
            code: body.code,
        })
        .await
    {
        Ok(tokens) => {
            let refresh_cookie = build_cookie(
                (REFRESH_COOKIE, tokens.refresh_token.clone()),
                Some(tokens.refresh_expires_in.num_seconds()),
            );

            Ok((jar.add(refresh_cookie), OkResponse(tokens.into())))
        }
        Err(e) => match e {
            LoginUserError::NotAuthenticated => Err(ErrResponse::Unauthorized(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct VerifyOtpPayload {
    #[schema(format = "email")]
    email: String,
    #[schema(min_length = 6, max_length = 6)]
    code: String,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Access token for autheticated user")]
pub(super) struct OkResponse(TokenData);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "Bad credentials")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, ApiError::bad_credentials()).into_response()
            }
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
