use axum::{Router, routing};
use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::api::ApiState;

mod exchange_code;
mod get_active_user;
mod logout_user;
mod redirect_oidc;
mod refresh_token;
mod send_otp;
mod verify_otp;

const AUTH_TAG: &str = "Auth";
const REFRESH_COOKIE: &str = "colette_refresh";
const CODE_VERIFIER_COOKIE: &str = "colette_code_verifier";
const STATE_COOKIE: &str = "colette_state";
const NONCE_COOKIE: &str = "colette_nonce";

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        User,
        TokenData,
        send_otp::SendOtpPayload,
        verify_otp::VerifyOtpPayload,
        exchange_code::CodePayload,
    )),
    paths(
        send_otp::handler,
        verify_otp::handler,
        get_active_user::handler,
        refresh_token::handler,
        logout_user::handler,
        redirect_oidc::handler,
        exchange_code::handler,
    )
)]
pub(crate) struct AuthApi;

impl AuthApi {
    pub(crate) fn public() -> Router<ApiState> {
        Router::new()
            .route("/send-otp", routing::post(send_otp::handler))
            .route("/verify-otp", routing::post(verify_otp::handler))
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

/// User account
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct User {
    /// Unique identifier of the user
    id: Uuid,
    /// Email address of the user
    #[schema(format = "email")]
    email: String,
    /// Whether the user's email has been verified
    verified: bool,
    /// Profile display name of the user
    #[schema(required)]
    display_name: Option<String>,
    /// Profile image URL of the user
    #[schema(required)]
    image_url: Option<Url>,
    /// Timestamp at which the user was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the user was last modified
    updated_at: DateTime<Utc>,
}

impl From<colette_authentication::User> for User {
    fn from(value: colette_authentication::User) -> Self {
        Self {
            id: value.id().as_inner(),
            email: value.email().email(),
            verified: value.verified(),
            display_name: value.display_name().map(|e| e.as_inner().to_owned()),
            image_url: value.image_url().cloned(),
            created_at: value.created_at(),
            updated_at: value.updated_at(),
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

impl From<colette_handler::TokenData> for TokenData {
    fn from(value: colette_handler::TokenData) -> Self {
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

impl From<colette_handler::TokenType> for TokenType {
    fn from(value: colette_handler::TokenType) -> Self {
        match value {
            colette_handler::TokenType::Bearer => TokenType::Bearer,
        }
    }
}
