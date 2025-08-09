use std::collections::HashMap;

use chrono::Utc;
use colette_http::HttpClient;
use http::{Request, header};
use http_body_util::BodyExt as _;
use jsonwebtoken::{DecodingKey, Header, Validation};
use url::Url;
use uuid::Uuid;

use super::{AuthConfig, TokenData, TokenType};
use crate::{
    Handler, RepositoryError, User,
    account::{AccountInsertParams, AccountRepository},
    auth::OIDC_PROVIDER,
    user::{UserInsertParams, UserRepository},
};

#[derive(Debug, Clone)]
pub struct ExchangeCodeCommand {
    pub code: String,
    pub code_verifier: String,
}

pub struct ExchangeCodeHandler {
    user_repository: Box<dyn UserRepository>,
    account_repository: Box<dyn AccountRepository>,
    http_client: Box<dyn HttpClient>,
    auth_config: AuthConfig,
}

impl ExchangeCodeHandler {
    pub fn new(
        user_repository: impl UserRepository,
        account_repository: impl AccountRepository,
        http_client: impl HttpClient,
        auth_config: AuthConfig,
    ) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            account_repository: Box::new(account_repository),
            http_client: Box::new(http_client),
            auth_config,
        }
    }
}

#[async_trait::async_trait]
impl Handler<ExchangeCodeCommand> for ExchangeCodeHandler {
    type Response = TokenData;
    type Error = ExchangeCodeError;

    async fn handle(&self, data: ExchangeCodeCommand) -> Result<Self::Response, Self::Error> {
        let Some(ref oidc_config) = self.auth_config.oidc else {
            return Err(ExchangeCodeError::NotAuthenticated);
        };

        let params = HashMap::from([
            ("grant_type", "authorization_code"),
            ("client_id", &oidc_config.client_id),
            ("code_verifier", &data.code_verifier),
            ("code", &data.code),
            ("redirect_uri", &oidc_config.redirect_uri),
        ]);

        let form_body = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let request = Request::post(oidc_config.token_endpoint.as_str())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form_body.into())
            .map_err(|e| ExchangeCodeError::Http(colette_http::Error::Http(e)))?;

        let resp = self.http_client.send(request).await?;
        let data = resp
            .into_body()
            .collect()
            .await
            .map_err(|e| ExchangeCodeError::Http(colette_http::Error::Client(e)))?
            .to_bytes();

        let oidc_token_data = serde_json::from_slice::<OidcTokenData>(&data)?;

        let claims = {
            let jwt_header = jsonwebtoken::decode_header(&oidc_token_data.id_token)?;
            let Some(kid) = jwt_header.kid else {
                return Err(ExchangeCodeError::MissingKid);
            };
            let Some(jwk) = oidc_config.jwk_set.find(&kid) else {
                return Err(ExchangeCodeError::MissingJwk);
            };

            let decoding_key = DecodingKey::from_jwk(jwk)?;

            let mut validation = Validation::new(jwt_header.alg);
            validation.set_issuer(&[&oidc_config.issuer]);
            validation.set_audience(&[&oidc_config.client_id]);

            let token_data = jsonwebtoken::decode::<OidcClaims>(
                &oidc_token_data.id_token,
                &decoding_key,
                &validation,
            )?;

            token_data.claims
        };

        let email = claims.email.unwrap();

        let user = match self
            .account_repository
            .find_by_sub_and_provider(claims.sub.clone(), OIDC_PROVIDER.into())
            .await?
        {
            Some(account) => match self.user_repository.find_by_id(account.user_id).await? {
                Some(user) => Ok(user),
                None => Err(ExchangeCodeError::NotAuthenticated),
            },
            None => {
                let user = match self.user_repository.find_by_email(email.clone()).await? {
                    Some(user) => {
                        self.account_repository
                            .insert(AccountInsertParams {
                                sub: claims.sub,
                                provider: OIDC_PROVIDER.into(),
                                password_hash: None,
                                user_id: user.id,
                            })
                            .await?;

                        user
                    }
                    None => {
                        let id = self
                            .user_repository
                            .insert(UserInsertParams {
                                email,
                                display_name: claims.name,
                                image_url: claims.picture,
                                sub: claims.sub,
                                provider: OIDC_PROVIDER.into(),
                                password_hash: None,
                            })
                            .await?;

                        self.user_repository.find_by_id(id).await?.unwrap()
                    }
                };

                Ok(user)
            }
        }?;

        let token_data = self.generate_tokens(user)?;

        Ok(token_data)
    }
}

impl ExchangeCodeHandler {
    fn generate_tokens(&self, user: User) -> Result<TokenData, ExchangeCodeError> {
        let now = Utc::now();

        let access_token = {
            let access_claims = Claims {
                iss: self.auth_config.jwt.issuer.clone(),
                sub: user.id,
                aud: self.auth_config.jwt.audience.clone(),
                exp: (now + self.auth_config.jwt.access_duration).timestamp(),
                iat: now.timestamp(),
            };

            jsonwebtoken::encode(
                &Header::default(),
                &access_claims,
                &self.auth_config.jwt.encoding_key,
            )?
        };

        let refresh_token = {
            let refresh_claims = Claims {
                iss: self.auth_config.jwt.issuer.clone(),
                sub: user.id,
                aud: self.auth_config.jwt.audience.clone(),
                exp: (now + self.auth_config.jwt.refresh_duration).timestamp(),
                iat: now.timestamp(),
            };

            jsonwebtoken::encode(
                &Header::default(),
                &refresh_claims,
                &self.auth_config.jwt.encoding_key,
            )?
        };

        Ok(TokenData {
            access_token,
            access_expires_in: self.auth_config.jwt.access_duration,
            refresh_token,
            refresh_expires_in: self.auth_config.jwt.refresh_duration,
            token_type: TokenType::default(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: Uuid,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OidcTokenData {
    pub id_token: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OidcClaims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<Url>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExchangeCodeError {
    #[error("Missing JWT key ID")]
    MissingKid,

    #[error("Missing JWK")]
    MissingJwk,

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Crypto(#[from] colette_util::CryptoError),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
