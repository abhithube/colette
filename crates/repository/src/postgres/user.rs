use colette_core::{
    User,
    auth::{Error, UserParams, UserRepository},
};
use colette_query::{IntoInsert, IntoSelect, user::UserInsert};
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

    async fn save(&self, data: &User) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = UserInsert {
            id: data.id,
            external_id: &data.external_id,
            email: data.email.as_deref(),
            display_name: data.display_name.as_deref(),
            picture_url: data.picture_url.as_ref().map(|e| e.as_str()),
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
        .into_insert()
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }
}

struct UserRow<'a>(&'a Row);

impl From<UserRow<'_>> for User {
    fn from(UserRow(value): UserRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            external_id: value.get("external_id"),
            email: value.get("email"),
            display_name: value.get("display_name"),
            picture_url: value
                .get::<_, Option<String>>("picture_url")
                .and_then(|e| e.parse().ok()),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}
