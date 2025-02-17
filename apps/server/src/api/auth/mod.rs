use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ApiState;

mod get_active_user;
mod login;
mod logout;
mod register;

pub const AUTH_TAG: &str = "Auth";

#[derive(OpenApi)]
#[openapi(components(schemas(register::Register, login::Login, User)))]
pub struct AuthApi;

impl AuthApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(AuthApi::openapi())
            .routes(routes!(register::handler))
            .routes(routes!(login::handler))
            .routes(routes!(get_active_user::handler))
            .routes(routes!(logout::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    #[schema(format = "email")]
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
