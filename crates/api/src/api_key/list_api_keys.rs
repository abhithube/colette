use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};

use super::{API_KEYS_TAG, ApiKey};
use crate::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listApiKeys",
    description = "List user API keys",
    tag = API_KEYS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, Error> {
    match state.api_key_service.list_api_keys(user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of API keys")]
    Ok(Paginated<ApiKey>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
