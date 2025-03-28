use axum::{
    Json,
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
    common::{AuthUser, BaseError, Error, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = ApiKeyCreate,
  responses(CreateResponse),
  operation_id = "createApiKey",
  description = "Create a API key",
  tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<ApiKeyCreate>,
) -> Result<impl IntoResponse, Error> {
    match state
        .api_key_service
        .create_api_key(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
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
pub struct ApiKeyCreated {
    pub id: Uuid,
    pub value: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
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

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created API key")]
    Created(ApiKeyCreated),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
