use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::api_key;

use super::{API_KEYS_TAG, ApiKey};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id, NonEmptyString},
};

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
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<ApiKeyUpdate>,
) -> Result<impl IntoResponse, Error> {
    match state
        .api_key_service
        .update_api_key(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            api_key::Error::Forbidden(_) => Ok(UpdateResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            api_key::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

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

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated API key")]
    Ok(ApiKey),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "API key not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
