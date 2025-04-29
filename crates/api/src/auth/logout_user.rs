use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::Cookie;
use torii::UserId;

use super::AUTH_TAG;
use crate::{
    ApiState,
    common::{ApiError, AuthUser, SESSION_COOKIE},
};

#[utoipa::path(
  post,
  path = "/logout",
  responses(OkResponse, ErrResponse),
  operation_id = "logoutUser",
  description = "Log out of user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, ErrResponse> {
    match state
        .auth
        .delete_sessions_for_user(&UserId::new(&user_id))
        .await
    {
        Ok(_) => {
            let removal_cookie = Cookie::build(SESSION_COOKIE)
                .removal()
                .path("/")
                .http_only(true)
                .secure(false)
                .build();

            Ok((
                [(header::SET_COOKIE, removal_cookie.to_string())],
                OkResponse,
            )
                .into_response())
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
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
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
