use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use email_address::EmailAddress;
use torii::ToriiError;

use super::{AUTH_TAG, User};
use crate::api::{
    ApiState,
    common::{BaseError, Error, NonEmptyString},
};

#[utoipa::path(
    post,
    path = "/register",
    request_body = Register,
    responses(RegisterResponse),
    operation_id = "registerUser",
    description = "Register a user account",
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<Register>,
) -> Result<RegisterResponse, Error> {
    match state
        .auth
        .register_user_with_password(body.email.as_str(), &String::from(body.password))
        .await
    {
        Ok(data) => Ok(RegisterResponse::Created(data.into())),
        Err(e) => match e {
            ToriiError::AuthError(message) => Ok(RegisterResponse::Conflict(BaseError { message })),
            _ => Err(Error::Auth(e)),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    #[schema(value_type = String, format = "email")]
    pub email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    pub password: NonEmptyString,
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
