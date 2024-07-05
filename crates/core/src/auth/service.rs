use super::{Error, Login, Register};
use crate::{
    profiles::{ProfileFindOneParams, ProfilesRepository},
    users::{self, UserCreateData, UserFindOneParams, UsersRepository},
    utils::password::PasswordHasher,
    Profile, User,
};

pub struct AuthService {
    users_repo: Box<dyn UsersRepository + Send + Sync>,
    profiles_repo: Box<dyn ProfilesRepository + Send + Sync>,
    hasher: Box<dyn PasswordHasher + Send + Sync>,
}

impl AuthService {
    pub fn new(
        users_repo: Box<dyn UsersRepository + Send + Sync>,
        profiles_repo: Box<dyn ProfilesRepository + Send + Sync>,
        hasher: Box<dyn PasswordHasher + Send + Sync>,
    ) -> Self {
        Self {
            users_repo,
            profiles_repo,
            hasher,
        }
    }

    pub async fn register(&self, data: Register<'_>) -> Result<User, Error> {
        let hashed = self
            .hasher
            .hash(data.password)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let data = UserCreateData {
            email: data.email,
            password: hashed.as_str(),
        };
        let user = self.users_repo.create(data).await?;

        Ok(user)
    }

    pub async fn login(&self, data: Login<'_>) -> Result<Profile, Error> {
        let params = UserFindOneParams { email: data.email };
        let user = self
            .users_repo
            .find_one(params)
            .await
            .map_err(|e| match e {
                users::Error::NotFound(_) => Error::NotAuthenticated,
                _ => Error::Users(e),
            })?;

        let valid = self
            .hasher
            .verify(data.password, user.password.as_str())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        let params = ProfileFindOneParams::Default {
            user_id: user.id.as_str(),
        };
        let profile = self.profiles_repo.find_one(params).await?;

        Ok(profile)
    }
}
