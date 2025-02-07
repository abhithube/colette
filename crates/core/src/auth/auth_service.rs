use colette_util::password;
use uuid::Uuid;

use super::Error;
use crate::{
    User,
    user::{self, UserCreateData, UserFindParams, UserRepository},
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
        let hashed = password::hash(&data.password);

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
            .find(UserFindParams::Email(data.email))
            .await
            .map_err(|e| match e {
                user::Error::NotFound(_) => Error::NotAuthenticated,
                _ => e.into(),
            })?;

        let valid = password::verify(&data.password, &user.password);
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
