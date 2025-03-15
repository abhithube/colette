use colette_util::password;
use uuid::Uuid;

use super::Error;
use crate::{
    User,
    user::{UserFindOne, UserRepository},
};

pub struct AuthService {
    repository: Box<dyn UserRepository>,
}

impl AuthService {
    pub fn new(repository: impl UserRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn register(&self, data: Register) -> Result<User, Error> {
        let user = User::builder()
            .email(data.email)
            .password_hash(password::hash(&data.password)?)
            .build();

        self.repository.save(&user).await?;

        Ok(user)
    }

    pub async fn login(&self, data: Login) -> Result<User, Error> {
        let Some(user) = self
            .repository
            .find_one(UserFindOne::Email(data.email))
            .await?
        else {
            return Err(Error::NotAuthenticated);
        };
        let Some(ref password_hash) = user.password_hash else {
            return Err(Error::NotAuthenticated);
        };

        let valid = password::verify(&data.password, password_hash)?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        Ok(user)
    }

    pub async fn get_active(&self, user_id: Uuid) -> Result<User, Error> {
        let Some(user) = self.repository.find_one(UserFindOne::Id(user_id)).await? else {
            return Err(Error::NotAuthenticated);
        };

        Ok(user)
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
