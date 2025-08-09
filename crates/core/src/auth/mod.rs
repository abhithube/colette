pub use build_authorization_url_handler::*;
use chrono::Duration;
pub use exchange_code_handler::*;
pub use get_user_handler::*;
use jsonwebtoken::{DecodingKey, EncodingKey, jwk::JwkSet};
pub use login_user_handler::*;
pub use refresh_access_token_handler::*;
pub use register_user_handler::*;
pub use validate_access_token_handler::*;

mod build_authorization_url_handler;
mod exchange_code_handler;
mod get_user_handler;
mod login_user_handler;
mod refresh_access_token_handler;
mod register_user_handler;
mod validate_access_token_handler;

const LOCAL_PROVIDER: &str = "local";
const OIDC_PROVIDER: &str = "oidc";

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
