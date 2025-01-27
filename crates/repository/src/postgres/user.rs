use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserFindParams, UserRepository},
    User,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: Pool<Postgres>,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params {
            UserFindParams::Id(id) => {
                crate::user::select_by_id(&self.pool, id)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Id(id)),
                        _ => Error::Unknown(e.into()),
                    })
            }
            UserFindParams::Email(email) => crate::user::select_by_email(&self.pool, &email)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Email(email)),
                    _ => Error::Unknown(e.into()),
                }),
        }
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresUserRepository {
    type Data = UserCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        crate::user::insert(&self.pool, data.email.clone(), data.password)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.email),
                _ => Error::Unknown(e.into()),
            })
    }
}

impl UserRepository for PostgresUserRepository {}
