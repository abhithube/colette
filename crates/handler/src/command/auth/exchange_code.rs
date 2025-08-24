use colette_authentication::{
    CustomProvider, DisplayName, Provider, SocialAccount, Sub, User, UserError, UserRepository,
};
use colette_common::RepositoryError;
use colette_jwt::{Claims, JwtManager};
use colette_oidc::OidcClient;
use email_address::EmailAddress;

use crate::{Handler, JwtConfig, OIDC_PROVIDER, TokenData, TokenType};

#[derive(Debug, Clone)]
pub struct ExchangeCodeCommand {
    pub code: String,
    pub code_verifier: String,
    pub nonce: String,
}

pub struct ExchangeCodeHandler<UR: UserRepository, OC: OidcClient, JM: JwtManager> {
    user_repository: UR,
    oidc_client: OC,
    jwt_manager: JM,
    jwt_config: JwtConfig,
}

impl<UR: UserRepository, OC: OidcClient, JM: JwtManager> ExchangeCodeHandler<UR, OC, JM> {
    pub fn new(
        user_repository: UR,
        oidc_client: OC,
        jwt_manager: JM,
        jwt_config: JwtConfig,
    ) -> Self {
        Self {
            user_repository,
            oidc_client,
            jwt_manager,
            jwt_config,
        }
    }
}

#[async_trait::async_trait]
impl<UR: UserRepository, OC: OidcClient, JM: JwtManager> Handler<ExchangeCodeCommand>
    for ExchangeCodeHandler<UR, OC, JM>
{
    type Response = TokenData;
    type Error = ExchangeCodeError;

    async fn handle(&self, cmd: ExchangeCodeCommand) -> Result<Self::Response, Self::Error> {
        let claims = self
            .oidc_client
            .exchange_code(cmd.code, cmd.code_verifier, cmd.nonce)
            .await?;

        let email = claims
            .email
            .unwrap()
            .parse::<EmailAddress>()
            .map_err(UserError::InvalidEmail)?;

        let user = match self.user_repository.find_by_email(email.clone()).await? {
            Some(user) => user,
            None => {
                let custom_provider =
                    CustomProvider::new(OIDC_PROVIDER.into()).map_err(UserError::SocialAccount)?;
                let sub = Sub::new(claims.sub).map_err(UserError::SocialAccount)?;

                let social_account = SocialAccount::new(Provider::Custom(custom_provider), sub);

                match self.user_repository.find_by_email(email.clone()).await? {
                    Some(mut user) => {
                        user.add_social_account(social_account)?;

                        self.user_repository.save(&user).await?;

                        user
                    }
                    None => {
                        let display_name =
                            claims.name.map(DisplayName::new_truncating).transpose()?;
                        let image_url = claims
                            .picture
                            .map(|e| e.parse().map_err(UserError::InvalidImageUrl))
                            .transpose()?;

                        let mut user = User::new(email, display_name, image_url);
                        user.add_social_account(social_account)?;

                        self.user_repository.save(&user).await?;

                        user
                    }
                }
            }
        };

        let access_token = self.jwt_manager.generate(Claims::new(
            user.id().as_inner().to_string(),
            self.jwt_config.access_duration,
        ))?;
        let refresh_token = self.jwt_manager.generate(Claims::new(
            user.id().as_inner().to_string(),
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
    User(#[from] UserError),

    #[error(transparent)]
    Oidc(#[from] colette_oidc::Error),

    #[error(transparent)]
    Jwt(#[from] colette_jwt::Error),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
