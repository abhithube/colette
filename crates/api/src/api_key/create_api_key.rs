use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::api_key;
use uuid::Uuid;

use super::API_KEYS_TAG;
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Json, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = ApiKeyCreate,
  responses(OkResponse, ErrResponse),
  operation_id = "createApiKey",
  description = "Create an API key",
  tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    AuthUser(user): AuthUser,
    Json(body): Json<ApiKeyCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .api_key_service
        .create_api_key(body.into(), user.id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ApiKeyCreate {
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
}

impl From<ApiKeyCreate> for api_key::ApiKeyCreate {
    fn from(value: ApiKeyCreate) -> Self {
        Self {
            title: value.title.into(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ApiKeyCreated {
    id: Uuid,
    value: String,
    title: String,
    created_at: DateTime<Utc>,
}

impl From<api_key::ApiKeyCreated> for ApiKeyCreated {
    fn from(value: api_key::ApiKeyCreated) -> Self {
        Self {
            id: value.id,
            value: value.value,
            title: value.title,
            created_at: value.created_at,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Created API key")]
pub(super) struct OkResponse(ApiKeyCreated);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

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
