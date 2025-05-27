use axum::{Router, routing};
use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use uuid::Uuid;

use super::{ApiState, common::Paginated};

mod create_api_key;
mod delete_api_key;
mod get_api_key;
mod list_api_keys;
mod update_api_key;

const API_KEYS_TAG: &str = "API Keys";

#[derive(OpenApi)]
#[openapi(
    components(schemas(ApiKey, Paginated<ApiKey>, create_api_key::ApiKeyCreate, create_api_key::ApiKeyCreated, update_api_key::ApiKeyUpdate)),
    paths(list_api_keys::handler, create_api_key::handler, get_api_key::handler, update_api_key::handler, delete_api_key::handler)
)]
pub(crate) struct ApiKeyApi;

impl ApiKeyApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_api_keys::handler))
            .route("/", routing::post(create_api_key::handler))
            .route("/{id}", routing::get(get_api_key::handler))
            .route("/{id}", routing::patch(update_api_key::handler))
            .route("/{id}", routing::delete(delete_api_key::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct ApiKey {
    id: Uuid,
    title: String,
    preview: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<colette_core::ApiKey> for ApiKey {
    fn from(value: colette_core::ApiKey) -> Self {
        Self {
            id: value.id,
            title: value.title,
            preview: value.preview,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
