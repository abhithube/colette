use colette_jwt::{Claims, JwtManager};
use colette_oidc::OidcClient;

use crate::{
    Handler, User,
    auth::{
        JwtConfig, OIDC_PROVIDER, Provider, SocialAccount, TokenData, TokenType, UserError,
        UserRepository,
    },
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct ExchangeCodeCommand {
    pub code: String,
    pub code_verifier: String,
    pub nonce: String,
}

pub struct ExchangeCodeHandler {
    user_repository: Box<dyn UserRepository>,
    oidc_client: Box<dyn OidcClient>,
    jwt_manager: Box<dyn JwtManager>,
    jwt_config: JwtConfig,
}

impl ExchangeCodeHandler {
    pub fn new(
        user_repository: impl UserRepository,
        oidc_client: impl OidcClient,
        jwt_manager: impl JwtManager,
        jwt_config: JwtConfig,
    ) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            oidc_client: Box::new(oidc_client),
            jwt_manager: Box::new(jwt_manager),
            jwt_config,
        }
    }
}

#[async_trait::async_trait]
impl Handler<ExchangeCodeCommand> for ExchangeCodeHandler {
    type Response = TokenData;
    type Error = ExchangeCodeError;

    async fn handle(&self, cmd: ExchangeCodeCommand) -> Result<Self::Response, Self::Error> {
        let claims = self
            .oidc_client
            .exchange_code(cmd.code, cmd.code_verifier, cmd.nonce)
            .await?;

        let email = claims.email.unwrap();

        let user = match self
            .user_repository
            .find_by_provider_and_sub(OIDC_PROVIDER.into(), claims.sub.clone())
            .await?
        {
            Some(user) => user,
            None => {
                let social_account =
                    SocialAccount::new(Provider::Other(OIDC_PROVIDER.into()), claims.sub);

                match self
                    .user_repository
                    .find_by_email(email.parse().unwrap())
                    .await?
                {
                    Some(mut user) => {
                        user.add_social_account(social_account)?;

                        self.user_repository.save(&user).await?;

                        user
                    }
                    None => {
                        let mut user = User::new(email, claims.name, claims.picture)?;
                        user.add_social_account(social_account)?;

                        self.user_repository.save(&user).await?;

                        user
                    }
                }
            }
        };

        let access_token = self.jwt_manager.generate(Claims::new(
            user.id().to_string(),
            self.jwt_config.access_duration,
        ))?;
        let refresh_token = self.jwt_manager.generate(Claims::new(
            user.id().to_string(),
            self.jwt_config.refresh_duration,
        ))?;

        Ok(TokenData {
            access_token,
            access_expires_in: self.jwt_config.access_duration,
            refresh_token,
            refresh_expires_in: self.jwt_config.refresh_duration,
            token_type: TokenType::Bearer,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExchangeCodeError {
    #[error(transparent)]
    Core(#[from] UserError),

    #[error(transparent)]
    Oidc(#[from] colette_oidc::Error),

    #[error(transparent)]
    Jwt(#[from] colette_jwt::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
