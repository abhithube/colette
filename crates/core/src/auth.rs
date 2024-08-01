use std::sync::Arc;

use crate::{
    common::Session,
    profiles::{self, ProfilesFindOneParams, ProfilesRepository},
    users::{self, UsersCreateData, UsersFindOneParams, UsersRepository},
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
    users_repo: Arc<dyn UsersRepository>,
    profiles_repo: Arc<dyn ProfilesRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl AuthService {
    pub fn new(
        users_repo: Arc<dyn UsersRepository>,
        profiles_repo: Arc<dyn ProfilesRepository>,
        hasher: Arc<dyn PasswordHasher>,
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

        self.users_repo
            .create_user(UsersCreateData {
                email: data.email,
                password: hashed,
            })
            .await
            .map_err(|e| e.into())
    }

    pub async fn login(&self, data: Login) -> Result<Profile, Error> {
        let user = self
            .users_repo
            .find_one_user(UsersFindOneParams::Email(data.email))
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

        self.profiles_repo
            .find_one_profile(ProfilesFindOneParams::Default { user_id: user.id })
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_active(&self, session: Session) -> Result<User, Error> {
        self.users_repo
            .find_one_user(UsersFindOneParams::Id(session.user_id))
            .await
            .map_err(|e| e.into())
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
