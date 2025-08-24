use colette_core::auth::OidcConfig;
use colette_oidc::{AuthorizationUrlData, OidcClient};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct BuildAuthorizationUrlQuery;

pub struct BuildAuthorizationUrlHandler<OC: OidcClient> {
    oidc_client: OC,
    oidc_config: OidcConfig,
}

impl<OC: OidcClient> BuildAuthorizationUrlHandler<OC> {
    pub fn new(oidc_client: OC, oidc_config: OidcConfig) -> Self {
        Self {
            oidc_client,
            oidc_config,
        }
    }
}

#[async_trait::async_trait]
impl<OC: OidcClient> Handler<BuildAuthorizationUrlQuery> for BuildAuthorizationUrlHandler<OC> {
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
