use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::Serialize;
use serde_json::{Map, Value};

pub trait JwtManager: Sync {
    fn generate(&self, claims: Claims) -> Result<String, Error>;

    fn verify(&self, token: &str) -> Result<Claims, Error>;
}

#[derive(Clone)]
pub struct JwtManagerImpl {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManagerImpl {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation: Validation::new(Algorithm::HS256),
        }
    }
}

impl JwtManager for JwtManagerImpl {
    fn generate(&self, claims: Claims) -> Result<String, Error> {
        let token = jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(Error::Encode)?;

        Ok(token)
    }

    fn verify(&self, token: &str) -> Result<Claims, Error> {
        let token_data =
            jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &self.validation)
                .map_err(Error::Decode)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
    iat: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl Claims {
    pub fn new(sub: String, expires_in: Duration) -> Self {
        let now = Utc::now();

        Self {
            sub: sub.clone(),
            exp: (now + expires_in).timestamp() as u64,
            iat: now.timestamp() as u64,
            iss: None,
            aud: None,
            extra: Map::new(),
        }
    }

    pub fn with_claim<T: Serialize>(mut self, key: String, value: T) -> Result<Self, Error> {
        let json_value = serde_json::to_value(value)?;
        self.extra.insert(key, json_value);

        Ok(self)
    }

    pub fn sub(&self) -> &str {
        &self.sub
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("encode: {0}")]
    Encode(jsonwebtoken::errors::Error),

    #[error("decode: {0}")]
    Decode(jsonwebtoken::errors::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
