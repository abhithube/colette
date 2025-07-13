use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::auth;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::ApiState;

mod exchange_code;
mod get_active_user;
mod login_user;
mod logout_user;
mod redirect_oidc;
mod refresh_token;
mod register_user;

const AUTH_TAG: &str = "Auth";
const REFRESH_COOKIE: &str = "colette_refresh";
const CODE_VERIFIER_COOKIE: &str = "colette_code_verifier";
const STATE_COOKIE: &str = "colette_state";

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        User,
        TokenData,
        register_user::RegisterPayload,
        login_user::LoginPayload,
        exchange_code::CodePayload
    )),
    paths(
        register_user::handler,
        login_user::handler,
        get_active_user::handler,
        refresh_token::handler,
        logout_user::handler,
        redirect_oidc::handler,
        exchange_code::handler
    )
)]
pub(crate) struct AuthApi;

impl AuthApi {
    pub(crate) fn public() -> Router<ApiState> {
        Router::new()
            .route("/register", routing::post(register_user::handler))
            .route("/login", routing::post(login_user::handler))
            .route("/token", routing::post(refresh_token::handler))
            .route("/oidc/redirect", routing::get(redirect_oidc::handler))
            .route("/oidc/code", routing::post(exchange_code::handler))
    }

    pub(crate) fn authenticated() -> Router<ApiState> {
        Router::new()
            .route("/@me", routing::get(get_active_user::handler))
            .route("/logout", routing::post(logout_user::handler))
    }
}

/// User account. Supports email/password and OIDC.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct User {
    /// Unique identifier of the user
    id: Uuid,
    /// Email address of the user
    #[schema(format = "email")]
    email: String,
    #[schema(required)]
    display_name: Option<String>,
    #[schema(required)]
    image_url: Option<Url>,
    /// Timestamp at which the user was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the user was last modified
    updated_at: DateTime<Utc>,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            display_name: value.display_name,
            image_url: value.image_url,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenData {
    access_token: String,
    token_type: TokenType,
    expires_in: i64,
}

impl From<auth::TokenData> for TokenData {
    fn from(value: auth::TokenData) -> Self {
        TokenData {
            access_token: value.access_token,
            token_type: value.token_type.into(),
            expires_in: value.access_expires_in.num_seconds(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, utoipa::ToSchema)]
pub enum TokenType {
    #[default]
    #[serde(rename = "bearer")]
    Bearer,
}

impl From<auth::TokenType> for TokenType {
    fn from(value: auth::TokenType) -> Self {
        match value {
            auth::TokenType::Bearer => TokenType::Bearer,
        }
    }
}
