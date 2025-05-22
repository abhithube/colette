use colette_http::HttpClient;
use http::{Request, header};
use http_body_util::BodyExt;
use jsonwebtoken::{DecodingKey, Validation, jwk::JwkSet};
use url::Url;
use uuid::Uuid;

use super::{Error, NotFoundError, User, UserRepository};

pub struct AuthService {
    repository: Box<dyn UserRepository>,
    http_client: Box<dyn HttpClient>,
    oidc_config: OidcConfig,
}

impl AuthService {
    pub fn new(
        repository: impl UserRepository,
        http_client: impl HttpClient,
        oidc_config: OidcConfig,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            http_client: Box::new(http_client),
            oidc_config,
        }
    }

    pub async fn validate_access_token(&self, access_token: &str) -> Result<User, Error> {
        let sub = {
            let jwt_header = jsonwebtoken::decode_header(access_token)?;
            let Some(kid) = jwt_header.kid else {
                return Err(Error::MissingKid);
            };
            let Some(jwk) = self.oidc_config.jwk_set.find(&kid) else {
                return Err(Error::MissingJwk);
            };

            let decoding_key = DecodingKey::from_jwk(jwk)?;
            let mut validation = Validation::new(jwt_header.alg);
            validation.set_audience(&["account"]);

            let token_data =
                jsonwebtoken::decode::<Claims>(access_token, &decoding_key, &validation)?;

            token_data.claims.sub
        };

        let user = match self.repository.find_by_external_id(sub).await? {
            Some(user) => user,
            None => {
                let request = Request::get(self.oidc_config.userinfo_endpoint.as_str())
                    .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
                    .body(Default::default())
                    .map_err(|e| Error::Http(colette_http::Error::Http(e)))?;

                let resp = self.http_client.send(request).await?;
                let data = resp
                    .into_body()
                    .collect()
                    .await
                    .map_err(|e| Error::Http(colette_http::Error::Client(e)))?
                    .to_bytes();

                let user_info = serde_json::from_slice::<UserInfo>(&data)?;

                let user = User::builder()
                    .external_id(user_info.sub)
                    .maybe_email(user_info.email)
                    .maybe_display_name(user_info.name)
                    .maybe_picture_url(user_info.picture)
                    .build();

                self.repository.save(&user).await?;

                user
            }
        };

        Ok(user)
    }

    pub async fn get_user(&self, query: UserGetQuery) -> Result<User, Error> {
        match query {
            UserGetQuery::Id(id) => {
                let Some(user) = self.repository.find_by_id(id).await? else {
                    return Err(Error::NotFound(NotFoundError::Id(id)));
                };

                Ok(user)
            }
            UserGetQuery::ExternalId(external_id) => {
                let Some(user) = self
                    .repository
                    .find_by_external_id(external_id.clone())
                    .await?
                else {
                    return Err(Error::NotFound(NotFoundError::ExternalId(external_id)));
                };

                Ok(user)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub userinfo_endpoint: Url,
    pub jwk_set: JwkSet,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Claims {
    sub: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<Url>,
}

#[derive(Debug, Clone)]
pub enum UserGetQuery {
    Id(Uuid),
    ExternalId(String),
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub external_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub picture_url: Option<Url>,
}

#[derive(Debug, Clone, Default)]
pub struct UserUpdate {
    pub display_name: Option<Option<String>>,
    pub picture_url: Option<Option<Url>>,
}
