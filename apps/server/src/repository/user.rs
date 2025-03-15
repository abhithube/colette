use chrono::{DateTime, Utc};
use colette_core::{
    User,
    user::{Error, UserFindOne, UserRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    user::{UserInsert, UserSelectOne},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

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
    async fn find_one(&self, key: UserFindOne) -> Result<Option<User>, Error> {
        let (sql, values) = match key {
            UserFindOne::Id(id) => UserSelectOne::Id(id),
            UserFindOne::Email(email) => UserSelectOne::Email(email),
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, data: &User) -> Result<(), Error> {
        let (sql, values) = UserInsert {
            id: data.id,
            email: &data.email,
            name: data.name.as_deref(),
            verified_at: None,
            password_hash: data.password_hash.as_deref(),
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(data.email.clone())
                }
                _ => Error::Database(e),
            })?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    verified_at: Option<DateTime<Utc>>,
    name: Option<String>,
    password_hash: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            verified_at: value.verified_at,
            name: value.name,
            password_hash: value.password_hash,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
