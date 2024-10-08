use colette_sql::user;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

#[derive(Debug, Clone, sqlx::FromRow)]
struct UserSelect {
    id: Uuid,
    email: String,
    password: String,
}

impl From<UserSelect> for colette_core::User {
    fn from(value: UserSelect) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
        }
    }
}

pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    email: Option<String>,
) -> sqlx::Result<colette_core::User> {
    let query = user::select(id, email);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
        .fetch_one(executor)
        .await
        .map(|e| e.into())
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    email: String,
    password: String,
) -> sqlx::Result<colette_core::User> {
    let query = user::insert(id, email, password);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
        .fetch_one(executor)
        .await
        .map(|e| e.into())
}
