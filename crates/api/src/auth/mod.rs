use axum::{Router, routing};
use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use super::ApiState;

mod get_active_user;

const AUTH_TAG: &str = "Auth";

#[derive(OpenApi)]
#[openapi(components(schemas(User)), paths(get_active_user::handler))]
pub(crate) struct AuthApi;

impl AuthApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new().route("/@me", routing::get(get_active_user::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct User {
    id: Uuid,
    external_id: String,
    #[schema(required, format = "email")]
    email: Option<String>,
    #[schema(required)]
    display_name: Option<String>,
    #[schema(required)]
    picture_url: Option<Url>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id,
            external_id: value.external_id,
            email: value.email,
            display_name: value.display_name,
            picture_url: value.picture_url,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
