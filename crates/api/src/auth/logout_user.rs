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
    common::{AuthUser, Error, SESSION_COOKIE},
};

#[utoipa::path(
  post,
  path = "/logout",
  responses(LogoutResponse),
  operation_id = "logoutUser",
  description = "Log out of user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, Error> {
    state
        .auth
        .delete_sessions_for_user(&UserId::new(&user_id))
        .await?;

    let removal_cookie = Cookie::build(SESSION_COOKIE)
        .removal()
        .path("/")
        .http_only(true)
        .secure(false)
        .build();

    Ok((
        [(header::SET_COOKIE, removal_cookie.to_string())],
        LogoutResponse::NoContent,
    )
        .into_response())
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum LogoutResponse {
    #[response(status = 204, description = "Successfully logged out")]
    NoContent,
}

impl IntoResponse for LogoutResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}
