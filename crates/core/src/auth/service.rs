use std::sync::Arc;

use crate::{
    password::PasswordHasher,
    users::{UserCreateData, UserFindOneParams, UsersRepository},
    User,
};

use super::{Error, LoginDto, RegisterDto};

pub struct AuthService {
    users_repo: Arc<dyn UsersRepository + Send + Sync>,
    hasher: Arc<dyn PasswordHasher + Send + Sync>,
}

impl AuthService {
    pub fn new(
        users_repo: Arc<dyn UsersRepository + Send + Sync>,
        hasher: Arc<dyn PasswordHasher + Send + Sync>,
    ) -> Self {
        Self { users_repo, hasher }
    }

    pub async fn register(&self, dto: RegisterDto) -> Result<User, Error> {
        let hashed = self
            .hasher
            .hash(&dto.password)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let data = UserCreateData {
            email: dto.email,
            password: hashed,
        };
        let user = self.users_repo.create(data).await?;

        Ok(user)
    }

    pub async fn login(&self, dto: LoginDto) -> Result<User, Error> {
        let params = UserFindOneParams { email: dto.email };
        let user = self.users_repo.find_one(params).await?;

        let valid = self
            .hasher
            .verify(&dto.password, &user.password)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if !valid {
            return Err(Error::NotAuthenticated);
        }

        Ok(user)
    }
}
