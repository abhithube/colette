use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use colette_core::auth;

use super::{AUTH_TAG, TokenData, build_refresh_cookie};
use crate::{
    ApiState,
    common::{ApiError, Json},
};

#[utoipa::path(
  post,
  path = "/code",
  request_body = CodePayload,
  responses(OkResponse, ErrResponse),
  operation_id = "exchangeCode",
  description = "Log in, and optionally register, a user from an OAuth authorization code",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<CodePayload>,
) -> Result<impl IntoResponse, ErrResponse> {
    match state.auth_service.exchange_code(body.into()).await {
        Ok(tokens) => {
            let cookie = build_refresh_cookie(&tokens.refresh_token, tokens.refresh_expires_in);

            Ok((
                [(header::SET_COOKIE, cookie.to_string())],
                OkResponse(tokens.into()),
            ))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CodePayload {
    code: String,
    code_verifier: String,
    nonce: String,
}

impl From<CodePayload> for auth::CodePayload {
    fn from(value: CodePayload) -> Self {
        auth::CodePayload {
            code: value.code,
            code_verifier: value.code_verifier,
            nonce: value.nonce,
        }
    }
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
