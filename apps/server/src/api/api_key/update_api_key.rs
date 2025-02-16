use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::api_key::{self, ApiKeyService};

use super::ApiKey;
use crate::api::common::{API_KEYS_TAG, BaseError, Error, Id, NonEmptyString, Session};

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
}

impl From<ApiKeyUpdate> for api_key::ApiKeyUpdate {
    fn from(value: ApiKeyUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
        }
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = ApiKeyUpdate,
    responses(UpdateResponse),
    operation_id = "updateApiKey",
    description = "Update an API key by ID",
    tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<ApiKeyService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<ApiKeyUpdate>,
) -> Result<impl IntoResponse, Error> {
    match service
        .update_api_key(id, body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            api_key::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated API key")]
    Ok(ApiKey),

    #[response(status = 404, description = "API key not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
