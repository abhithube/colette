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
                sqlx::query_file_as!(User, "queries/users/select_by_id.sql", id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Id(id)),
                        _ => Error::Database(e),
                    })
            }
            UserFindParams::Email(email) => {
                sqlx::query_file_as!(User, "queries/users/select_by_email.sql", email)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Email(email)),
                        _ => Error::Database(e),
                    })
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresUserRepository {
    type Data = UserCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let email = String::from(data.email);

        sqlx::query_file_scalar!("queries/users/insert.sql", email, data.password)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(email),
                _ => Error::Database(e),
            })
    }
}

impl UserRepository for PostgresUserRepository {}
