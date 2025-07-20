use chrono::{DateTime, Utc};
use colette_core::{
    User,
    user::{Error, UserParams, UserRepository},
};
use colette_query::{IntoSelect, user::UserSelect};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

use crate::postgres::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn query(&self, params: UserParams) -> Result<Vec<User>, Error> {
        let (sql, values) = UserSelect {
            id: params.id,
            email: params.email.as_deref(),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    display_name: Option<String>,
    image_url: Option<DbUrl>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            display_name: value.display_name,
            image_url: value.image_url.map(|e| e.0),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
