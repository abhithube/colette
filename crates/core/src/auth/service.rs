use std::sync::Arc;

use crate::{
    profiles::{ProfileFindOneParams, ProfilesRepository},
    users::{UserCreateData, UserFindOneParams, UsersRepository},
    utils::password::PasswordHasher,
    Profile, User,
};

use super::{Error, LoginDto, RegisterDto};

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

    pub async fn register(&self, dto: RegisterDto) -> Result<User, Error> {
        let hashed = self
            .hasher
            .hash(&dto.password)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let data = UserCreateData {
            email: dto.email.as_str(),
            password: hashed.as_str(),
        };
        let user = self.users_repo.create(data).await?;

        Ok(user)
    }

    pub async fn login(&self, dto: LoginDto) -> Result<Profile, Error> {
        let params = UserFindOneParams {
            email: dto.email.as_str(),
        };
        let user = self.users_repo.find_one(params).await?;

        let valid = self
            .hasher
            .verify(&dto.password, &user.password)
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
