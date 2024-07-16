use colette_core::{
    users::{Error, UserCreateData, UserFindOneParams, UsersRepository},
    User,
};
use sqlx::PgPool;

use crate::queries::{
    profiles::{self, InsertParams},
    users,
};

pub struct UsersPostgresRepository {
    pool: PgPool,
}

impl UsersPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UsersRepository for UsersPostgresRepository {
    async fn find_one(&self, params: UserFindOneParams) -> Result<User, Error> {
        let user = users::select_by_email(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.email),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(user)
    }

    async fn create(&self, data: UserCreateData) -> Result<User, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let user = users::insert(&mut *tx, (&data).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.email),
                _ => Error::Unknown(e.into()),
            })?;

        profiles::insert(&mut *tx, InsertParams::default_with_user(&user.id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(user)
    }
}
