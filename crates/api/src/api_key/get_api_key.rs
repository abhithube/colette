use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    api_key::{GetApiKeyError, GetApiKeyQuery},
};

use super::{API_KEYS_TAG, ApiKey};
use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(OkResponse, ErrResponse),
    operation_id = "getApiKey",
    description = "Get an API key by ID",
    tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .get_api_key
        .handle(GetApiKeyQuery { id, user_id })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            GetApiKeyError::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            GetApiKeyError::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "API key by ID")]
pub(super) struct OkResponse(ApiKey);

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

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "API key not found")]
    NotFound(ApiError),

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
