use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ApiState;

mod get_active_user;
mod login_user;
mod logout_user;
mod register_user;

const AUTH_TAG: &str = "Auth";

#[derive(OpenApi)]
#[openapi(components(schemas(register_user::Register, login_user::Login, User)))]
pub(crate) struct AuthApi;

impl AuthApi {
    pub(crate) fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(AuthApi::openapi())
            .routes(routes!(register_user::handler))
            .routes(routes!(login_user::handler))
            .routes(routes!(get_active_user::handler))
            .routes(routes!(logout_user::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct User {
    id: String,
    #[schema(format = "email")]
    email: String,
    verified_at: Option<DateTime<Utc>>,
    name: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<torii::User> for User {
    fn from(value: torii::User) -> Self {
        Self {
            id: value.id.into_inner(),
            email: value.email,
            verified_at: value.email_verified_at,
            name: value.name,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
