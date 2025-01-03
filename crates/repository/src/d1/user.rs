use std::sync::Arc;

use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserFindParams, UserRepository},
    User,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::{D1Binder, D1Error};

#[derive(Clone)]
pub struct D1UserRepository {
    db: Arc<D1Database>,
}

impl D1UserRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1UserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params {
            UserFindParams::Id(id) => {
                let (sql, values) =
                    crate::user::select(Some(id), None).build_d1(SqliteQueryBuilder);

                let Some(user) = super::first::<User>(&self.db, sql, values, None)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(NotFoundError::Id(id)));
                };

                Ok(user)
            }
            UserFindParams::Email(email) => {
                let (sql, values) =
                    crate::user::select(None, Some(email.clone())).build_d1(SqliteQueryBuilder);

                let Some(user) = super::first::<User>(&self.db, sql, values, None)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(NotFoundError::Email(email)));
                };

                Ok(user)
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for D1UserRepository {
    type Data = UserCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();

        let (sql, values) = crate::user::insert(Some(id), data.email.clone(), data.password)
            .build_d1(SqliteQueryBuilder);

        super::run(&self.db, sql, values)
            .await
            .map_err(|e| match e.into() {
                D1Error::UniqueConstraint => Error::Conflict(data.email),
                e => Error::Unknown(e.into()),
            })?;

        Ok(id)
    }
}

impl UserRepository for D1UserRepository {}
