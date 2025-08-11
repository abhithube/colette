use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, jwk::JwkSet};

pub const LOCAL_PROVIDER: &str = "local";
pub const OIDC_PROVIDER: &str = "oidc";

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: Vec<String>,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub access_duration: Duration,
    pub refresh_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub client_id: String,
    pub issuer: String,
    pub redirect_uri: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwk_set: JwkSet,
    pub scope: String,
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
