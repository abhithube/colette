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
    user,
};
use email_address::EmailAddress;

use super::User;
use crate::common::{AUTH_TAG, BaseError, Error};

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    #[schema(value_type = String, format = "email")]
    pub email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    pub password: NonEmptyString,
}

impl From<Register> for auth::Register {
    fn from(value: Register) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = Register,
    responses(RegisterResponse),
    operation_id = "register",
    description = "Register a user account",
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<AuthService>>,
    Json(body): Json<Register>,
) -> Result<RegisterResponse, Error> {
    match service.register(body.into()).await {
        Ok(data) => Ok(RegisterResponse::Created(data.into())),
        Err(e) => match e {
            auth::Error::Users(user::Error::Conflict(_)) => {
                Ok(RegisterResponse::Conflict(BaseError {
                    message: e.to_string(),
                }))
            }
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum RegisterResponse {
    #[response(status = 201, description = "Registered user")]
    Created(User),

    #[response(status = 409, description = "Email already registered")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for RegisterResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
