use axum::{
    Extension,
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
    common::{ApiError, ConnectionInfo, Json, NonEmptyString, SESSION_COOKIE},
};

#[utoipa::path(
  post,
  path = "/login",
  request_body = Login,
  responses(OkResponse, ErrResponse),
  operation_id = "loginUser",
  description = "Login to a user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Extension(connection_info): Extension<ConnectionInfo>,
    Json(body): Json<Login>,
) -> Result<impl IntoResponse, ErrResponse> {
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
                OkResponse(user.into()),
            )
                .into_response())
        }
        Err(ToriiError::AuthError(message)) => Err(ErrResponse::Unauthorized(ApiError { message })),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct Login {
    #[schema(value_type = String, format = "email")]
    email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    password: NonEmptyString,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Logged in user")]
pub(super) struct OkResponse(User);

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
