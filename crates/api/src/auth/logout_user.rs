use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::{AUTH_TAG, REFRESH_COOKIE};
use crate::common::ApiError;

#[utoipa::path(
  post,
  path = "/logout",
  responses(OkResponse, ErrResponse),
  operation_id = "logoutUser",
  description = "Logout the active user",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(jar: CookieJar) -> Result<impl IntoResponse, ErrResponse> {
    let Some(mut refresh_cookie) = jar.get(REFRESH_COOKIE).cloned() else {
        return Err(ErrResponse::Unauthorized(ApiError::forbidden()));
    };

    refresh_cookie.set_path("/");

    Ok((jar.remove(refresh_cookie), OkResponse))
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully logged out")]
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
            Self::Unauthorized(_) => (StatusCode::FORBIDDEN, ApiError::forbidden()).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
        }
    }
}
