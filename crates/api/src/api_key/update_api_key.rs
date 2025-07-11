use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::api_key;

use super::{API_KEYS_TAG, ApiKey};
use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Json, NonEmptyString, Path},
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = ApiKeyUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updateApiKey",
    description = "Update an API key by ID",
    tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<ApiKeyUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .api_key_service
        .update_api_key(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            api_key::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            api_key::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Details regarding the existing API key to update
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ApiKeyUpdate {
    /// Human-readable name for the API key to update, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    title: Option<NonEmptyString>,
}

impl From<ApiKeyUpdate> for api_key::ApiKeyUpdate {
    fn from(value: ApiKeyUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Updated API key")]
pub(super) struct OkResponse(ApiKey);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "API key not found")]
    NotFound(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
