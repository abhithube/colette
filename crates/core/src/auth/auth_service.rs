use colette_util::password;
use email_address::EmailAddress;
use uuid::Uuid;

use super::Error;
use crate::{
    common::NonEmptyString,
    user::{self, UserCreateData, UserFindParams, UserRepository},
    User,
};

pub struct AuthService {
    user_repository: Box<dyn UserRepository>,
}

impl AuthService {
    pub fn new(user_repository: impl UserRepository) -> Self {
        Self {
            user_repository: Box::new(user_repository),
        }
    }

    pub async fn register(&self, data: Register) -> Result<User, Error> {
        let hashed = password::hash(&String::from(data.password));

        let id = self
            .user_repository
            .create(UserCreateData {
                email: data.email,
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

        let valid = password::verify(&String::from(data.password), &user.password);
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

#[derive(Clone, Debug)]
pub struct Register {
    pub email: EmailAddress,
    pub password: NonEmptyString,
}

#[derive(Clone, Debug)]
pub struct Login {
    pub email: EmailAddress,
    pub password: NonEmptyString,
}
