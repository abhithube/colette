use colette_util::password;
use uuid::Uuid;

use super::Error;
use crate::{
    User,
    account::{self, AccountCreateParams, AccountFindParams, AccountRepository},
    common::TransactionManager,
    user::{UserCreateParams, UserFindParams, UserRepository},
};

pub struct AuthService {
    user_repository: Box<dyn UserRepository>,
    account_repository: Box<dyn AccountRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl AuthService {
    pub fn new(
        user_repository: impl UserRepository,
        account_repository: impl AccountRepository,
        tx_manager: impl TransactionManager,
    ) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            account_repository: Box::new(account_repository),
            tx_manager: Box::new(tx_manager),
        }
    }

    pub async fn register(&self, data: Register) -> Result<User, Error> {
        let id = Uuid::new_v4();
        let hashed = password::hash(&data.password)?;

        let tx = self.tx_manager.begin().await?;

        self.user_repository
            .create_user(
                &*tx,
                UserCreateParams {
                    id,
                    email: data.email.clone(),
                    ..Default::default()
                },
            )
            .await?;

        self.account_repository
            .create_account(
                &*tx,
                AccountCreateParams {
                    user_id: id,
                    provider_id: "local".into(),
                    account_id: data.email.clone(),
                    password_hash: Some(hashed),
                },
            )
            .await?;

        self.user_repository
            .find_user(UserFindParams { id })
            .await
            .map_err(Error::Users)
    }

    pub async fn login(&self, data: Login) -> Result<User, Error> {
        let account = self
            .account_repository
            .find_account(AccountFindParams {
                provider_id: "local".into(),
                account_id: data.email,
            })
            .await
            .map_err(|e| match e {
                account::Error::NotFound(_) => Error::NotAuthenticated,
                _ => e.into(),
            })?;
        let Some(password_hash) = account.password_hash else {
            return Err(Error::NotAuthenticated);
        };

        let valid = password::verify(&data.password, &password_hash)?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        self.user_repository
            .find_user(UserFindParams { id: account.id })
            .await
            .map_err(Error::Users)
    }

    pub async fn get_active(&self, user_id: Uuid) -> Result<User, Error> {
        self.user_repository
            .find_user(UserFindParams { id: user_id })
            .await
            .map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct Register {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Login {
    pub email: String,
    pub password: String,
}

// #[derive(Debug, Clone)]
// pub struct UserCreateData {
//     pub email: String,
//     pub account: AccountCreateData,
// }

// #[derive(Debug, Clone)]
// pub enum AccountCreateData {
//     Local {
//         password_hash: String,
//     },
//     Oidc {
//         provider_id: String,
//         provider_account_id: String,
//     },
// }
