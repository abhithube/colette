use oauth2::{
    AuthorizationCode, ClientId, ConfigurationError, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, HttpClientError, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    RequestTokenError, Scope, StandardErrorResponse, basic::BasicErrorResponseType,
};
use openidconnect::{
    ClaimsVerificationError, DiscoveryError, IssuerUrl, Nonce, TokenResponse as _,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use reqwest::Client as ReqwestClient;
use url::{ParseError, Url};

#[async_trait::async_trait]
pub trait OidcClient: Send + Sync + 'static {
    fn build_authorization_url(&self, scopes: Vec<String>) -> AuthorizationUrlData;

    async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: String,
        nonce: String,
    ) -> Result<Claims, Error>;
}

#[derive(Clone)]
pub struct OidcClientImpl {
    oidc_client: CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointMaybeSet,
        EndpointMaybeSet,
    >,
    http_client: ReqwestClient,
}

impl OidcClientImpl {
    pub async fn init(config: OidcConfig, http_client: ReqwestClient) -> Result<Self, Error> {
        let issuer_url = IssuerUrl::new(config.issuer_url)?;

        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer_url, &http_client).await?;

        let client_id = ClientId::new(config.client_id);
        let redirect_url = RedirectUrl::new(config.redirect_uri)?;

        let oidc_client = CoreClient::from_provider_metadata(provider_metadata, client_id, None)
            .set_redirect_uri(redirect_url);

        Ok(Self {
            oidc_client,
            http_client,
        })
    }
}

#[async_trait::async_trait]
impl OidcClient for OidcClientImpl {
    fn build_authorization_url(&self, scopes: Vec<String>) -> AuthorizationUrlData {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut request = self.oidc_client.authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        for scope in scopes {
            request = request.add_scope(Scope::new(scope));
        }

        let (auth_url, csrf_token, nonce) = request.set_pkce_challenge(pkce_challenge).url();

        AuthorizationUrlData {
            auth_url,
            csrf_token: csrf_token.into_secret(),
            nonce: nonce.secret().to_owned(),
            code_verifier: pkce_verifier.into_secret(),
        }
    }

    async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: String,
        nonce: String,
    ) -> Result<Claims, Error> {
        let token_response = self
            .oidc_client
            .exchange_code(AuthorizationCode::new(code))?
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
            .request_async(&self.http_client)
            .await?;

        let id_token = token_response.id_token().ok_or(Error::NoIdToken)?;

        let claims = id_token.claims(&self.oidc_client.id_token_verifier(), &Nonce::new(nonce))?;

        Ok(Claims {
            sub: claims.subject().to_string(),
            email: claims.email().map(|e| e.to_string()),
            verified: claims.email_verified().unwrap_or_default(),
            name: claims
                .name()
                .and_then(|e| e.get(None).map(|e| e.to_string())),
            picture: claims
                .picture()
                .and_then(|e| e.get(None).map(|e| e.to_string())),
        })
    }
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct AuthorizationUrlData {
    pub auth_url: Url,
    pub csrf_token: String,
    pub nonce: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ID token not provided")]
    NoIdToken,

    #[error(transparent)]
    Discovery(#[from] DiscoveryError<HttpClientError<reqwest::Error>>),

    #[error(transparent)]
    Token(
        #[from]
        RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ),

    #[error(transparent)]
    Claims(#[from] ClaimsVerificationError),

    #[error(transparent)]
    Configuration(#[from] ConfigurationError),

    #[error(transparent)]
    Url(#[from] ParseError),
}
