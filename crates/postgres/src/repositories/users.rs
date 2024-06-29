use async_trait::async_trait;
use colette_core::{
    users::{CreateData, Error, FindOneParams, Repository},
    User,
};
use sqlx::PgPool;

use crate::queries::users;

pub struct UsersRepository<'a> {
    pool: &'a PgPool,
}

#[async_trait]
impl Repository for UsersRepository<'_> {
    async fn find_one(&self, params: FindOneParams) -> Result<User, Error> {
        let email = params.email.clone();

        users::select_by_email(self.pool, params.into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(email),
                _ => Error::Unknown(e.into()),
            })?;

        todo!()
    }

    async fn create(&self, data: CreateData) -> Result<User, Error> {
        let email = data.email.clone();

        let user = users::insert(self.pool, data.into())
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(email),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(user)
    }
}
