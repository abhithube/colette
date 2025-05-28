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

/// User account. A new user is created if the "sub" field in the OIDC access token does not match an existing user.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct User {
    /// Unique identifier of the user
    id: Uuid,
    /// Unique identifier of the user from the external identity server
    external_id: String,
    /// Email address of the user from the external identity server
    #[schema(required, format = "email")]
    email: Option<String>,
    #[schema(required)]
    /// Display name of the user
    display_name: Option<String>,
    #[schema(required)]
    /// Profile picture URL of the user
    picture_url: Option<Url>,
    /// Timestamp at which the user was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the user was last modified
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
