use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    api_key::{ApiKeyCursor, ListApiKeysError, ListApiKeysQuery},
};

use crate::{
    ApiState,
    api_key::{API_KEYS_TAG, ApiKey},
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
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
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<ApiKeyCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_api_keys
        .handle(ListApiKeysQuery {
            cursor,
            limit: Some(PAGINATION_LIMIT),
            user_id,
        })
        .await
    {
        Ok(api_keys) => {
            let data = api_keys
                .try_into()
                .map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
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

impl From<ListApiKeysError> for ErrResponse {
    fn from(value: ListApiKeysError) -> Self {
        ErrResponse::InternalServerError(value.into())
    }
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
