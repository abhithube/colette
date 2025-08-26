use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use colette_handler::{ExchangeCodeCommand, Handler as _};

use crate::api::{
    ApiState,
    auth::{AUTH_TAG, CODE_VERIFIER_COOKIE, NONCE_COOKIE, REFRESH_COOKIE, STATE_COOKIE, TokenData},
    common::{ApiError, ApiErrorCode, Json, build_cookie},
};

#[utoipa::path(
  post,
  path = "/oidc/code",
  request_body = CodePayload,
  responses(OkResponse, ErrResponse),
  operation_id = "exchangeCode",
  description = "Log in, and optionally register, a user from an OAuth authorization code",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    jar: CookieJar,
    Json(body): Json<CodePayload>,
) -> Result<impl IntoResponse, ErrResponse> {
    let Some(mut code_verifier_cookie) = jar.get(CODE_VERIFIER_COOKIE).cloned() else {
        return Err(ErrResponse::Conflict(ApiError {
            code: ApiErrorCode::Conflict,
            message: "Missing code_verifier cookie".into(),
        }));
    };
    code_verifier_cookie.set_path("/");

    let Some(mut state_cookie) = jar.get(STATE_COOKIE).cloned() else {
        return Err(ErrResponse::Conflict(ApiError {
            code: ApiErrorCode::Conflict,
            message: "Missing state cookie".into(),
        }));
    };
    state_cookie.set_path("/");

    if state_cookie.value() != body.state {
        return Err(ErrResponse::Conflict(ApiError {
            code: ApiErrorCode::Conflict,
            message: "Invalid state".into(),
        }));
    }

    let Some(mut nonce_cookie) = jar.get(NONCE_COOKIE).cloned() else {
        return Err(ErrResponse::Conflict(ApiError {
            code: ApiErrorCode::Conflict,
            message: "Missing nonce cookie".into(),
        }));
    };
    nonce_cookie.set_path("/");

    match state
        .exchange_code
        .unwrap()
        .handle(ExchangeCodeCommand {
            code: body.code,
            code_verifier: code_verifier_cookie.value().into(),
            nonce: nonce_cookie.value().into(),
        })
        .await
    {
        Ok(tokens) => {
            let refresh_cookie = build_cookie(
                (REFRESH_COOKIE, tokens.refresh_token.clone()),
                Some(tokens.refresh_expires_in.num_seconds()),
            );

            Ok((
                jar.remove(code_verifier_cookie)
                    .remove(state_cookie)
                    .add(refresh_cookie),
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
    state: String,
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
    #[response(status = StatusCode::CONFLICT, description = "Missing OAuth cookies")]
    Conflict(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
