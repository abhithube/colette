use colette_core::{
    ApiKey,
    api_key::{ApiKeyParams, ApiKeyRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    api_key::{ApiKeyDelete, ApiKeyInsert},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteApiKeyRepository {
    pool: Pool,
}

impl SqliteApiKeyRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for SqliteApiKeyRepository {
    async fn query(&self, params: ApiKeyParams) -> Result<Vec<ApiKey>, Error> {
        let client = self.pool.get().await?;

        let api_keys = client
            .interact(move |conn| {
                let (sql, values) = params.into_select().build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<ApiKey>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(api_keys)
    }

    async fn save(&self, data: &ApiKey) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let (sql, values) = ApiKeyInsert {
                    id: data.id,
                    lookup_hash: &data.lookup_hash,
                    verification_hash: &data.verification_hash,
                    title: &data.title,
                    preview: &data.preview,
                    user_id: data.user_id,
                    created_at: data.created_at,
                    updated_at: data.updated_at,
                }
                .into_insert()
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = ApiKeyDelete { id }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for ApiKey {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            lookup_hash: value.get_unwrap("lookup_hash"),
            verification_hash: value.get_unwrap("verification_hash"),
            title: value.get_unwrap("title"),
            preview: value.get_unwrap("preview"),
            user_id: value.get_unwrap("user_id"),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
        }
    }
}
