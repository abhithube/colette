use colette_oidc::{AuthorizationUrlData, OidcClient};

use crate::{Handler, auth::OidcConfig};

#[derive(Debug, Clone)]
pub struct BuildAuthorizationUrlQuery;

pub struct BuildAuthorizationUrlHandler {
    oidc_client: Box<dyn OidcClient>,
    oidc_config: OidcConfig,
}

impl BuildAuthorizationUrlHandler {
    pub fn new(oidc_client: impl OidcClient, oidc_config: OidcConfig) -> Self {
        Self {
            oidc_client: Box::new(oidc_client),
            oidc_config,
        }
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
        let data = self
            .oidc_client
            .build_authorization_url(self.oidc_config.scopes.clone());

        Ok(data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildAuthorizationUrlError {}
