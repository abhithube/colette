use colette_core::{
    User,
    user::{Error, UserParams, UserRepository},
};
use colette_query::{IntoSelect, user::UserSelect};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: Pool,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn query(&self, params: UserParams) -> Result<Vec<User>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = UserSelect {
            id: params.id,
            email: params.email.as_deref(),
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let users = client.query_prepared::<User>(&sql, &values).await?;

        Ok(users)
    }
}

impl From<PgRow<'_>> for User {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            email: value.get("email"),
            display_name: value.get("display_name"),
            image_url: value
                .get::<_, Option<String>>("image_url")
                .and_then(|e| e.parse().ok()),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}
