use colette_core::{
    User,
    common::Findable,
    user::{Error, UserFindParams, UserRepository},
};
use sqlx::{Pool, Postgres};

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
        sqlx::query_file_as!(User, "queries/users/select.sql", params.id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })
    }
}

impl UserRepository for PostgresUserRepository {}
