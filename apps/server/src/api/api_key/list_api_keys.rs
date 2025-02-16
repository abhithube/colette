use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use colette_core::api_key::ApiKeyService;

use super::ApiKey;
use crate::api::common::{API_KEYS_TAG, Error, Paginated, Session};

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
    State(service): State<Arc<ApiKeyService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.list_api_keys(session.user_id).await {
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
