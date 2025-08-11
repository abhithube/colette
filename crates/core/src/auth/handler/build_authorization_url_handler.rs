use colette_util::{base64_url_encode, random_generate, sha256_hash};
use url::Url;

use crate::{Handler, auth::AuthConfig, common::RepositoryError};

#[derive(Debug, Clone)]
pub struct BuildAuthorizationUrlQuery {}

pub struct BuildAuthorizationUrlHandler {
    auth_config: AuthConfig,
}

impl BuildAuthorizationUrlHandler {
    pub fn new(auth_config: AuthConfig) -> Self {
        Self { auth_config }
    }
}

#[async_trait::async_trait]
impl Handler<BuildAuthorizationUrlQuery> for BuildAuthorizationUrlHandler {
    type Response = AuthorizationUrlData;
    type Error = BuildAuthorizationUrlError;

    async fn handle(
        &self,
        _query: BuildAuthorizationUrlQuery,
    ) -> Result<Self::Response, Self::Error> {
        let oidc_config = self
            .auth_config
            .oidc
            .as_ref()
            .ok_or_else(|| BuildAuthorizationUrlError::NotAuthenticated)?;

        let code_verifier = base64_url_encode(&random_generate(43));
        let code_challenge = base64_url_encode(&sha256_hash(&code_verifier));
        let state = base64_url_encode(&random_generate(32));

        let params = vec![
            ("response_type", "code"),
            ("client_id", &oidc_config.client_id),
            ("redirect_uri", &oidc_config.redirect_uri),
            ("scope", &oidc_config.scope),
            ("code_challenge_method", "S256"),
            ("code_challenge", &code_challenge),
            ("state", &state),
        ];

        let authorization_url =
            Url::parse_with_params(&oidc_config.authorization_endpoint, params).unwrap();

        Ok(AuthorizationUrlData {
            url: authorization_url.into(),
            code_verifier,
            state,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizationUrlData {
    pub url: String,
    pub code_verifier: String,
    pub state: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildAuthorizationUrlError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
