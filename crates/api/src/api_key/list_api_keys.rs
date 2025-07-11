use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::api_key;

use super::{API_KEYS_TAG, ApiKey};
use crate::{
    ApiState,
    common::{ApiError, Auth, Paginated, Query},
};

#[utoipa::path(
    get,
    path = "",
    params(ApiKeyListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listApiKeys",
    description = "List user API keys",
    tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<ApiKeyListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .api_key_service
        .list_api_keys(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct ApiKeyListQuery {
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
}

impl From<ApiKeyListQuery> for api_key::ApiKeyListQuery {
    fn from(value: ApiKeyListQuery) -> Self {
        Self {
            cursor: value.cursor,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of API keys")]
pub(super) struct OkResponse(Paginated<ApiKey>);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

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
