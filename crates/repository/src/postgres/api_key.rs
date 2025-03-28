use colette_core::{
    ApiKey,
    api_key::{ApiKeyParams, ApiKeyRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    api_key::{ApiKeyDelete, ApiKeyInsert},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresApiKeyRepository {
    pool: Pool,
}

impl PostgresApiKeyRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn query(&self, params: ApiKeyParams) -> Result<Vec<ApiKey>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows.iter().map(|e| ApiKeyRow(e).into()).collect())
    }

    async fn save(&self, data: &ApiKey) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = ApiKeyInsert {
            id: data.id,
            lookup_hash: &data.lookup_hash,
            verification_hash: &data.verification_hash,
            title: &data.title,
            preview: &data.preview,
            user_id: &data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
        .into_insert()
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = ApiKeyDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }
}

struct ApiKeyRow<'a>(&'a Row);

impl From<ApiKeyRow<'_>> for ApiKey {
    fn from(ApiKeyRow(value): ApiKeyRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            lookup_hash: value.get("lookup_hash"),
            verification_hash: value.get("verification_hash"),
            title: value.get("title"),
            preview: value.get("preview"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}
