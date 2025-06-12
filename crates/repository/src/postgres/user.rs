use colette_core::{
    User,
    user::{Error, UserParams, UserRepository},
};
use colette_query::IntoSelect;
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;

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

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows.iter().map(|e| UserRow(e).into()).collect())
    }
}

struct UserRow<'a>(&'a Row);

impl From<UserRow<'_>> for User {
    fn from(UserRow(value): UserRow<'_>) -> Self {
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
