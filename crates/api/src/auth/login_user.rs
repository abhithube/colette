use axum::{
    Extension, Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::Cookie;
use chrono::Utc;
use email_address::EmailAddress;
use time::Duration;
use torii::ToriiError;

use super::{AUTH_TAG, User};
use crate::{
    ApiState,
    common::{BaseError, ConnectionInfo, Error, NonEmptyString, SESSION_COOKIE},
};

#[utoipa::path(
  post,
  path = "/login",
  request_body = Login,
  responses(LoginResponse),
  operation_id = "loginUser",
  description = "Login to a user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Extension(connection_info): Extension<ConnectionInfo>,
    Json(body): Json<Login>,
) -> Result<impl IntoResponse, Error> {
    match state
        .auth
        .login_user_with_password(
            body.email.as_str(),
            &String::from(body.password),
            connection_info.user_agent,
            connection_info.ip_address,
        )
        .await
    {
        Ok((user, session)) => {
            let cookie = Cookie::build((SESSION_COOKIE, session.token.into_inner()))
                .path("/")
                .http_only(true)
                .secure(false)
                .max_age(Duration::seconds(
                    session.expires_at.timestamp() - Utc::now().timestamp(),
                ))
                .build();

            Ok((
                [(header::SET_COOKIE, cookie.to_string())],
                LoginResponse::Ok(user.into()),
            )
                .into_response())
        }
        Err(e) => match e {
            ToriiError::AuthError(message) => {
                Ok(LoginResponse::Unauthorized(BaseError { message }).into_response())
            }
            _ => Err(Error::Auth(e)),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    #[schema(value_type = String, format = "email")]
    pub email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    pub password: NonEmptyString,
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum LoginResponse {
    #[response(status = 200, description = "Logged in user")]
    Ok(User),

    #[response(status = 401, description = "Bad credentials")]
    Unauthorized(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
