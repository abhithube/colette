use colette_util::argon2_hash;
use url::Url;

use crate::{
    Handler, RepositoryError,
    auth::LOCAL_PROVIDER,
    user::{UserInsertParams, UserRepository},
};

#[derive(Debug, Clone)]
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub image_url: Option<Url>,
}

pub struct RegisterUserHandler {
    user_repository: Box<dyn UserRepository>,
}

impl RegisterUserHandler {
    pub fn new(user_repository: impl UserRepository) -> Self {
        Self {
            user_repository: Box::new(user_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<RegisterUserCommand> for RegisterUserHandler {
    type Response = ();
    type Error = RegisterUserError;

    async fn handle(&self, cmd: RegisterUserCommand) -> Result<Self::Response, Self::Error> {
        let password_hash = argon2_hash(&cmd.password)?;

        self.user_repository
            .insert(UserInsertParams {
                email: cmd.email.clone(),
                display_name: cmd.display_name,
                image_url: cmd.image_url,
                sub: cmd.email,
                provider: LOCAL_PROVIDER.into(),
                password_hash: Some(password_hash),
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterUserError {
    #[error(transparent)]
    Crypto(#[from] colette_util::CryptoError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
