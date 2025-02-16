use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::api_key::{self, ApiKeyService};
use uuid::Uuid;

use crate::api::common::{API_KEYS_TAG, BaseError, Error, NonEmptyString, Session};

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
    State(service): State<Arc<ApiKeyService>>,
    session: Session,
    Json(body): Json<ApiKeyCreate>,
) -> Result<impl IntoResponse, Error> {
    match service.create_api_key(body.into(), session.user_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            api_key::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created API key")]
    Created(ApiKeyCreated),

    #[response(status = 409, description = "API key already exists")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
