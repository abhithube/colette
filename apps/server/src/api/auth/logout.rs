use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::api::common::{AUTH_TAG, BaseError, Error};

#[utoipa::path(
  post,
  path = "/logout",
  responses(LogoutResponse),
  operation_id = "logout",
  description = "Log out of user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(session_store: tower_sessions::Session) -> Result<LogoutResponse, Error> {
    session_store.delete().await?;

    Ok(LogoutResponse::NoContent)
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum LogoutResponse {
    #[response(status = 204, description = "Successfully logged out")]
    NoContent,

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for LogoutResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
