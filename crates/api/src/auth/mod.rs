use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::auth;
use email_address::EmailAddress;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated};

mod create_pat;
mod delete_pat;
mod exchange_code;
mod get_active_user;
mod get_pat;
mod list_pats;
mod logout_user;
mod redirect_oidc;
mod refresh_token;
mod send_otp;
mod update_pat;
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
        PersonalAccessToken, Paginated<PersonalAccessToken>, create_pat::PatCreate, create_pat::PatCreated, update_pat::PatUpdate
    )),
    paths(
        send_otp::handler,
        verify_otp::handler,
        get_active_user::handler,
        refresh_token::handler,
        logout_user::handler,
        redirect_oidc::handler,
        exchange_code::handler,
        list_pats::handler, create_pat::handler, get_pat::handler, update_pat::handler, delete_pat::handler
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
            .route("/pats", routing::get(list_pats::handler))
            .route("/pats", routing::post(create_pat::handler))
            .route("/pats/{id}", routing::get(get_pat::handler))
            .route("/pats/{id}", routing::patch(update_pat::handler))
            .route("/pats/{id}", routing::delete(delete_pat::handler))
    }
}

/// User account
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct User {
    /// Unique identifier of the user
    id: Uuid,
    /// Email address of the user
    #[schema(value_type = String, format = "email")]
    email: EmailAddress,
    /// Whether the user's email has been verified
    verified: bool,
    /// Profile display name of the user
    #[schema(required)]
    display_name: Option<String>,
    /// Profile image URL of the user
    #[schema(required)]
    image_url: Option<Url>,
    /// Generated PATs
    personal_access_tokens: Vec<PersonalAccessToken>,
    /// Timestamp at which the user was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the user was last modified
    updated_at: DateTime<Utc>,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id().as_inner(),
            email: value.email().to_owned(),
            verified: value.verified(),
            display_name: value.display_name().map(Into::into),
            image_url: value.image_url().cloned(),
            personal_access_tokens: value
                .personal_access_tokens()
                .to_owned()
                .into_iter()
                .map(Into::into)
                .collect(),
            created_at: value.created_at(),
            updated_at: value.updated_at(),
        }
    }
}

/// PAT, used for long-lived token access to the API
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PersonalAccessToken {
    id: Uuid,
    lookup_hash: String,
    verification_hash: String,
    title: String,
    preview: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<auth::PersonalAccessToken> for PersonalAccessToken {
    fn from(value: auth::PersonalAccessToken) -> Self {
        Self {
            id: value.id().as_inner(),
            lookup_hash: value.lookup_hash().as_inner().to_owned(),
            verification_hash: value.verification_hash().as_inner().to_owned(),
            title: value.title().as_inner().to_owned(),
            preview: value.preview().as_inner().to_owned(),
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
