use chrono::{Duration, Utc};
use colette_http::HttpClient;
use http::Request;
use http_body_util::BodyExt as _;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, jwk::JwkSet};
use serde_json::json;
use url::Url;
use uuid::Uuid;

use super::Error;
use crate::{
    Account,
    account::AccountRepository,
    user::{User, UserRepository},
};

const LOCAL_PROVIDER: &str = "local";
const OIDC_PROVIDER: &str = "oidc";

pub struct AuthService {
    user_repository: Box<dyn UserRepository>,
    account_repository: Box<dyn AccountRepository>,
    http_client: Box<dyn HttpClient>,
    config: AuthConfig,
}

impl AuthService {
    pub fn new(
        user_repository: impl UserRepository,
        account_repository: impl AccountRepository,
        http_client: impl HttpClient,
        config: AuthConfig,
    ) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            account_repository: Box::new(account_repository),
            http_client: Box::new(http_client),
            config,
        }
    }

    pub async fn register_user(&self, data: RegisterPayload) -> Result<User, Error> {
        let password_hash = colette_util::argon2_hash(&data.password)?;

        let user = User::builder()
            .email(data.email.clone())
            .maybe_display_name(data.display_name)
            .maybe_image_url(data.image_url)
            .build();

        let account = Account::builder()
            .sub(data.email)
            .provider(LOCAL_PROVIDER.into())
            .password_hash(password_hash)
            .user_id(user.id)
            .user(user.clone())
            .build();

        self.account_repository.save(&account).await?;

        Ok(user)
    }

    pub async fn login_user(&self, data: LoginPayload) -> Result<TokenData, Error> {
        let account = match self
            .account_repository
            .find_by_sub_and_provider(data.email, LOCAL_PROVIDER.into())
            .await?
        {
            Some(account) => Ok(account),
            None => Err(Error::NotAuthenticated),
        }?;

        let Some(password_hash) = account.password_hash else {
            return Err(Error::NotAuthenticated);
        };

        let valid = colette_util::argon2_verify(&data.password, &password_hash)?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        let user = match self.user_repository.find_by_id(account.user_id).await? {
            Some(user) => Ok(user),
            None => Err(Error::NotAuthenticated),
        }?;

        let token_data = self.generate_tokens(user)?;

        Ok(token_data)
    }

    pub async fn validate_access_token(&self, access_token: &str) -> Result<Claims, Error> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.jwt.issuer]);
        validation.set_audience(&self.config.jwt.audience);

        let token_data = jsonwebtoken::decode::<Claims>(
            access_token,
            &self.config.jwt.decoding_key,
            &validation,
        )?;

        Ok(token_data.claims)
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User, Error> {
        let Some(user) = self.user_repository.find_by_id(id).await? else {
            return Err(Error::NotAuthenticated);
        };

        Ok(user)
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenData, Error> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.jwt.issuer]);
        validation.set_audience(&self.config.jwt.audience);

        let token_data = jsonwebtoken::decode::<Claims>(
            refresh_token,
            &self.config.jwt.decoding_key,
            &validation,
        )?;

        let user = match self
            .user_repository
            .find_by_id(token_data.claims.sub)
            .await?
        {
            Some(user) => Ok(user),
            None => Err(Error::NotAuthenticated),
        }?;

        let token_data = self.generate_tokens(user)?;

        Ok(token_data)
    }

    pub async fn exchange_code(&self, data: CodePayload) -> Result<TokenData, Error> {
        let Some(ref oidc_config) = self.config.oidc else {
            return Err(Error::NotAuthenticated);
        };

        let body = serde_json::to_vec(&json!({
            "grant_type": "authorization_code",
            "client_id": oidc_config.client_id,
            "code_verifier": data.code_verifier,
            "code": data.code,
            "redirect_uri": oidc_config.redirect_uri
        }))?;

        let request = Request::post(oidc_config.token_endpoint.as_str())
            .body(body.into())
            .map_err(|e| Error::Http(colette_http::Error::Http(e)))?;

        let resp = self.http_client.send(request).await?;
        let data = resp
            .into_body()
            .collect()
            .await
            .map_err(|e| Error::Http(colette_http::Error::Client(e)))?
            .to_bytes();

        let oidc_token_data = serde_json::from_slice::<OidcTokenData>(&data)?;

        let claims = {
            let jwt_header = jsonwebtoken::decode_header(&oidc_token_data.id_token)?;
            let Some(kid) = jwt_header.kid else {
                return Err(Error::MissingKid);
            };
            let Some(jwk) = oidc_config.jwk_set.find(&kid) else {
                return Err(Error::MissingJwk);
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

        let user = match self
            .account_repository
            .find_by_sub_and_provider(claims.sub.clone(), OIDC_PROVIDER.into())
            .await?
        {
            Some(account) => match self.user_repository.find_by_id(account.user_id).await? {
                Some(user) => Ok(user),
                None => Err(Error::NotAuthenticated),
            },
            None => {
                let user = User::builder()
                    .email(claims.email.unwrap())
                    .maybe_display_name(claims.name)
                    .maybe_image_url(claims.picture)
                    .build();

                let account = Account::builder()
                    .sub(claims.sub)
                    .provider(OIDC_PROVIDER.into())
                    .user_id(user.id)
                    .user(user.clone())
                    .build();

                self.account_repository.save(&account).await?;

                Ok(user)
            }
        }?;

        let token_data = self.generate_tokens(user)?;

        Ok(token_data)
    }

    fn generate_tokens(&self, user: User) -> Result<TokenData, Error> {
        let now = Utc::now();

        let access_token = {
            let access_claims = Claims {
                iss: self.config.jwt.issuer.clone(),
                sub: user.id,
                aud: self.config.jwt.audience.clone(),
                exp: (now + self.config.jwt.access_duration).timestamp(),
                iat: now.timestamp(),
            };

            jsonwebtoken::encode(
                &Header::default(),
                &access_claims,
                &self.config.jwt.encoding_key,
            )?
        };

        let refresh_token = {
            let refresh_claims = Claims {
                iss: self.config.jwt.issuer.clone(),
                sub: user.id,
                aud: self.config.jwt.audience.clone(),
                exp: (now + self.config.jwt.refresh_duration).timestamp(),
                iat: now.timestamp(),
            };

            jsonwebtoken::encode(
                &Header::default(),
                &refresh_claims,
                &self.config.jwt.encoding_key,
            )?
        };

        Ok(TokenData {
            access_token,
            access_expires_in: self.config.jwt.access_duration,
            refresh_token,
            refresh_expires_in: self.config.jwt.refresh_duration,
            token_type: TokenType::default(),
        })
    }
}

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
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwk_set: JwkSet,
}

#[derive(Debug, Clone)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub image_url: Option<Url>,
}

#[derive(Debug, Clone)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct CodePayload {
    pub code: String,
    pub code_verifier: String,
    pub nonce: String,
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
