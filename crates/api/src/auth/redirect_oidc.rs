use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;

use super::{AUTH_TAG, CODE_VERIFIER_COOKIE, STATE_COOKIE};
use crate::{
    ApiState,
    common::{ApiError, build_cookie},
};

#[utoipa::path(
  get,
  path = "/oidc/redirect",
  responses(OkResponse, ErrResponse),
  operation_id = "redirectOidc",
  description = "Initiate the OIDC flow by redirecting to the authorization URL",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ErrResponse> {
    match state.auth_service.build_authorization_url().await {
        Ok(data) => {
            let code_verifier_cookie =
                build_cookie((CODE_VERIFIER_COOKIE, data.code_verifier), None);
            let state_cookie = build_cookie((STATE_COOKIE, data.state), None);

            Ok((
                jar.add(code_verifier_cookie).add(state_cookie),
                Redirect::to(&data.url),
            ))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::SEE_OTHER, description = "Redirect to OIDC authorization endpoint")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::SEE_OTHER.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
        }
    }
}
