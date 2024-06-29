use async_trait::async_trait;
use colette_core::{
    users::{CreateData, Error, Repository},
    User,
};
use sqlx::PgPool;

use crate::queries::users::insert;

pub struct UsersRepository<'a> {
    pool: &'a PgPool,
}

#[async_trait]
impl Repository for UsersRepository<'_> {
    async fn create(&self, data: CreateData) -> Result<User, Error> {
        let email = data.email.clone();

        let user = insert(self.pool, data.into()).await.map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(email),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(user)
    }
}
