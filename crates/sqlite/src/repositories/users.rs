use async_trait::async_trait;
use colette_core::{
    users::{Error, NotFoundError, UsersCreateData, UsersFindOneParams, UsersRepository},
    User,
};
use sqlx::SqlitePool;

use crate::queries::{self, profiles::InsertParams};

pub struct UsersSqliteRepository {
    pool: SqlitePool,
}

impl UsersSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UsersRepository for UsersSqliteRepository {
    async fn find_one(&self, params: UsersFindOneParams) -> Result<User, Error> {
        let user = queries::users::select_by_email(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Email(params.email)),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(user)
    }

    async fn create(&self, data: UsersCreateData) -> Result<User, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let user = queries::users::insert(&mut *tx, (&data).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.email),
                _ => Error::Unknown(e.into()),
            })?;

        queries::profiles::insert(&mut *tx, InsertParams::default_with_user(user.id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(user)
    }
}
