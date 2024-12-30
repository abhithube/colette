use colette_util::PasswordHasher;
use email_address::EmailAddress;
use uuid::Uuid;

use crate::{
    common::NonEmptyString,
    user::{self, UserCreateData, UserFindParams, UserRepository},
    User,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Register {
    pub email: EmailAddress,
    pub password: NonEmptyString,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Login {
    pub email: EmailAddress,
    pub password: NonEmptyString,
}

#[derive(Clone)]
pub struct AuthService {
    user_repository: Box<dyn UserRepository>,
    password_hasher: Box<dyn PasswordHasher>,
}

impl AuthService {
    pub fn new(user_repository: impl UserRepository, password_hasher: impl PasswordHasher) -> Self {
        Self {
            user_repository: Box::new(user_repository),
            password_hasher: Box::new(password_hasher),
        }
    }

    pub async fn register(&self, data: Register) -> Result<User, Error> {
        let hashed = self.password_hasher.hash(&String::from(data.password))?;

        let id = self
            .user_repository
            .create(UserCreateData {
                email: data.email.into(),
                password: hashed,
            })
            .await
            .map_err(Error::Users)?;

        self.user_repository
            .find(UserFindParams::Id(id))
            .await
            .map_err(Error::Users)
    }

    pub async fn login(&self, data: Login) -> Result<User, Error> {
        let user = self
            .user_repository
            .find(UserFindParams::Email(String::from(data.email)))
            .await
            .map_err(|e| match e {
                user::Error::NotFound(_) => Error::NotAuthenticated,
                _ => e.into(),
            })?;

        let valid = self
            .password_hasher
            .verify(&String::from(data.password), &user.password)?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        Ok(user)
    }

    pub async fn get_active(&self, user_id: Uuid) -> Result<User, Error> {
        self.user_repository
            .find(UserFindParams::Id(user_id))
            .await
            .map_err(|e| e.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Users(#[from] user::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
