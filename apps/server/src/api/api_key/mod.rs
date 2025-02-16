use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_core::api_key::ApiKeyService;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::common::Paginated;

mod create_api_key;
mod delete_api_key;
mod get_api_key;
mod list_api_keys;
mod update_api_key;

pub const API_KEYS_TAG: &str = "API Keys";

#[derive(OpenApi)]
#[openapi(components(schemas(ApiKey, Paginated<ApiKey>, create_api_key::ApiKeyCreate, create_api_key::ApiKeyCreated, update_api_key::ApiKeyUpdate)))]
pub struct ApiKeyApi;

impl ApiKeyApi {
    pub fn router() -> OpenApiRouter<ApiKeyState> {
        OpenApiRouter::with_openapi(ApiKeyApi::openapi())
            .routes(routes!(list_api_keys::handler, create_api_key::handler))
            .routes(routes!(
                get_api_key::handler,
                update_api_key::handler,
                delete_api_key::handler
            ))
    }
}

#[derive(Clone, axum::extract::FromRef)]
pub struct ApiKeyState {
    pub service: Arc<ApiKeyService>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    pub id: Uuid,
    pub value_preview: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

impl From<colette_core::ApiKey> for ApiKey {
    fn from(value: colette_core::ApiKey) -> Self {
        Self {
            id: value.id,
            value_preview: value.value_preview,
            title: value.title,
            created_at: value.created_at,
        }
    }
}
