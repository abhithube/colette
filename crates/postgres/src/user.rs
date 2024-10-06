use sea_query::{Expr, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    CreatedAt,
    UpdatedAt,
}

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
    let query = Query::select()
        .columns([User::Id, User::Email, User::Password])
        .from(User::Table)
        .and_where_option(id.map(|e| Expr::col((User::Table, User::Id)).eq(e)))
        .and_where_option(email.map(|e| Expr::col((User::Table, User::Email)).eq(e)))
        .order_by((User::Table, User::Email), Order::Asc)
        .to_owned();

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
    let query = Query::insert()
        .into_table(User::Table)
        .columns([User::Id, User::Email, User::Password])
        .values_panic([id.into(), email.into(), password.into()])
        .returning_all()
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
        .fetch_one(executor)
        .await
        .map(|e| e.into())
}
