use colette_core::{
    User,
    common::Transaction,
    user::{Error, UserCreateParams, UserFindParams, UserRepository},
};
use colette_query::{IntoInsert, IntoSelect};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};

use super::common::parse_timestamp;

#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    pool: Pool<Sqlite>,
}

impl SqliteUserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_user(&self, params: UserFindParams) -> Result<User, Error> {
        let id = params.id;

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Database(e),
            })?;

        Ok(row.into())
    }

    async fn create_user(
        &self,
        tx: &dyn Transaction,
        params: UserCreateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;
        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    display_name: Option<String>,
    created_at: i32,
    updated_at: i32,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            email: value.email,
            display_name: value.display_name,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
