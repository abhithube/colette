use std::sync::Arc;

use crate::{
    profiles,
    profiles::{ProfileFindOneParams, ProfilesRepository},
    users::{self, UserCreateData, UserFindOneParams, UsersRepository},
    utils::password::PasswordHasher,
    Profile, User,
};

#[derive(Clone, Debug)]
pub struct Register {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct Login {
    pub email: String,
    pub password: String,
}

pub struct AuthService {
    users_repo: Arc<dyn UsersRepository + Send + Sync>,
    profiles_repo: Arc<dyn ProfilesRepository + Send + Sync>,
    hasher: Arc<dyn PasswordHasher + Send + Sync>,
}

impl AuthService {
    pub fn new(
        users_repo: Arc<dyn UsersRepository + Send + Sync>,
        profiles_repo: Arc<dyn ProfilesRepository + Send + Sync>,
        hasher: Arc<dyn PasswordHasher + Send + Sync>,
    ) -> Self {
        Self {
            users_repo,
            profiles_repo,
            hasher,
        }
    }

    pub async fn register(&self, data: Register) -> Result<User, Error> {
        let hashed = self
            .hasher
            .hash(&data.password)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let user = self
            .users_repo
            .create(UserCreateData {
                email: data.email,
                password: hashed,
            })
            .await?;

        Ok(user)
    }

    pub async fn login(&self, data: Login) -> Result<Profile, Error> {
        let user = self
            .users_repo
            .find_one(UserFindOneParams { email: data.email })
            .await
            .map_err(|e| match e {
                users::Error::NotFound(_) => Error::NotAuthenticated,
                _ => Error::Users(e),
            })?;

        let valid = self
            .hasher
            .verify(&data.password, &user.password)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        let profile = self
            .profiles_repo
            .find_one(ProfileFindOneParams::Default { user_id: user.id })
            .await?;

        Ok(profile)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Profiles(#[from] profiles::Error),

    #[error(transparent)]
    Users(#[from] users::Error),

    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
