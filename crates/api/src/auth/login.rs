use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    auth::{self, AuthService},
    common::NonEmptyString,
};
use email_address::EmailAddress;

use super::User;
use crate::common::{AUTH_TAG, BaseError, Error, SESSION_KEY, Session};

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    #[schema(value_type = String, format = "email")]
    pub email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    pub password: NonEmptyString,
}

impl From<Login> for auth::Login {
    fn from(value: Login) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

#[utoipa::path(
  post,
  path = "/login",
  request_body = Login,
  responses(LoginResponse),
  operation_id = "login",
  description = "Login to a user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    Json(body): Json<Login>,
) -> Result<LoginResponse, Error> {
    match service.login(body.into()).await {
        Ok(data) => {
            let session = Session { user_id: data.id };
            session_store.insert(SESSION_KEY, session).await?;

            Ok(LoginResponse::Ok(data.into()))
        }
        Err(e) => match e {
            auth::Error::NotAuthenticated => Ok(LoginResponse::Unauthorized(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

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
