use chrono::Duration;

use crate::auth::Provider;

pub const LOCAL_PROVIDER: &str = "local";
pub const OIDC_PROVIDER: &str = "oidc";

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: Vec<u8>,
    pub access_duration: Duration,
    pub refresh_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TokenData {
    pub access_token: String,
    pub access_expires_in: Duration,
    pub refresh_token: String,
    pub refresh_expires_in: Duration,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Default)]
pub enum TokenType {
    #[default]
    Bearer,
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("already connected to provider {0} with sub {1}")]
    DuplicateAccount(Provider, String),

    #[error("created too many OTP codes")]
    TooManyOtpCodes,

    #[error("duplicate OTP code")]
    DuplicateOtpCode,

    #[error("invalid OTP code")]
    InvalidOtpCode,

    #[error("already used OTP code")]
    AlreadyUsedOtpCode,

    #[error(transparent)]
    InvalidEmail(#[from] email_address::Error),

    #[error(transparent)]
    InvalidImageUrl(#[from] url::ParseError),
}
