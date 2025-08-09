use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{Handler as _, auth::RegisterUserCommand};
use email_address::EmailAddress;
use url::Url;

use super::AUTH_TAG;
use crate::{
    ApiState,
    common::{ApiError, Json, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "/register",
  request_body = RegisterPayload,
  responses(OkResponse, ErrResponse),
  operation_id = "registerUser",
  description = "Register a new user account",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Json(body): Json<RegisterPayload>,
) -> Result<impl IntoResponse, ErrResponse> {
    match state
        .register_user
        .handle(RegisterUserCommand {
            email: body.email.to_string(),
            password: body.password.into(),
            display_name: body.display_name.map(Into::into),
            image_url: body.image_url,
        })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct RegisterPayload {
    #[schema(value_type = String, format = "email")]
    email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    password: NonEmptyString,
    #[schema(value_type = Option<String>, min_length = 1)]
    display_name: Option<NonEmptyString>,
    image_url: Option<Url>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully created user")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::CONFLICT, description = "Email already registered")]
    Conflict(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
