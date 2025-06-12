use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::auth;
use email_address::EmailAddress;
use url::Url;

use super::{AUTH_TAG, User};
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
    match state.auth_service.register_user(body.into()).await {
        Ok(user) => Ok(OkResponse(user.into())),
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

impl From<RegisterPayload> for auth::RegisterPayload {
    fn from(value: RegisterPayload) -> Self {
        auth::RegisterPayload {
            email: value.email.into(),
            password: value.password.into(),
            display_name: value.display_name.map(Into::into),
            image_url: value.image_url,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Created user")]
pub(super) struct OkResponse(User);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
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
