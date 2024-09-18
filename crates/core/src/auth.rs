use std::sync::Arc;

use colette_util::PasswordHasher;
use email_address::EmailAddress;
use uuid::Uuid;

use crate::{
    common::NonEmptyString,
    profile::{self, ProfileIdOrDefaultParams, ProfileRepository},
    user::{self, UserCreateData, UserIdParams, UserRepository},
    Profile, User,
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SwitchProfile {
    pub id: Uuid,
}

pub struct AuthService {
    user_repository: Arc<dyn UserRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
}

impl AuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
    ) -> Self {
        Self {
            user_repository,
            profile_repository,
            password_hasher,
        }
    }

    pub async fn register(&self, data: Register) -> Result<User, Error> {
        let hashed = self.password_hasher.hash(&String::from(data.password))?;

        self.user_repository
            .create(UserCreateData {
                email: data.email.into(),
                password: hashed,
            })
            .await
            .map_err(Error::Users)
    }

    pub async fn login(&self, data: Login) -> Result<Profile, Error> {
        let user = self
            .user_repository
            .find(UserIdParams::Email(String::from(data.email)))
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

        self.profile_repository
            .find(ProfileIdOrDefaultParams {
                id: None,
                user_id: user.id,
            })
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_active(&self, user_id: Uuid) -> Result<User, Error> {
        self.user_repository
            .find(UserIdParams::Id(user_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn switch_profile(
        &self,
        data: SwitchProfile,
        user_id: Uuid,
    ) -> Result<Profile, Error> {
        self.profile_repository
            .find(ProfileIdOrDefaultParams {
                id: Some(data.id),
                user_id,
            })
            .await
            .map_err(|e| match e {
                profile::Error::NotFound(_) => Error::NotAuthenticated,
                _ => e.into(),
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Profiles(#[from] profile::Error),

    #[error(transparent)]
    Users(#[from] user::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
