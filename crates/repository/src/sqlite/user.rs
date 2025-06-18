use colette_core::{
    User,
    user::{Error, UserParams, UserRepository},
};
use colette_query::IntoSelect;
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    pool: Pool,
}

impl SqliteUserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for SqliteUserRepository {
    async fn query(&self, params: UserParams) -> Result<Vec<User>, Error> {
        let client = self.pool.get().await?;

        let users = client
            .interact(move |conn| {
                let (sql, values) = params.into_select().build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<User>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(users)
    }
}

impl From<SqliteRow<'_>> for User {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            email: value.get_unwrap("email"),
            display_name: value.get_unwrap("display_name"),
            image_url: value
                .get_unwrap::<_, Option<String>>("image_url")
                .and_then(|e| e.parse().ok()),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
        }
    }
}
