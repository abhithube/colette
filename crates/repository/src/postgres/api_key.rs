use chrono::{DateTime, Utc};
use colette_core::{
    ApiKey,
    api_key::{ApiKeyParams, ApiKeyRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    api_key::{ApiKeyDelete, ApiKeyInsert, ApiKeySelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresApiKeyRepository {
    pool: PgPool,
}

impl PostgresApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn query(&self, params: ApiKeyParams) -> Result<Vec<ApiKey>, Error> {
        let (sql, values) = ApiKeySelect {
            id: params.id,
            lookup_hash: params.lookup_hash.as_deref(),
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, ApiKeyRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &ApiKey) -> Result<(), Error> {
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
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = ApiKeyDelete { id }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ApiKeyRow {
    id: Uuid,
    lookup_hash: String,
    verification_hash: String,
    title: String,
    preview: String,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(value: ApiKeyRow) -> Self {
        Self {
            id: value.id,
            lookup_hash: value.lookup_hash,
            verification_hash: value.verification_hash,
            title: value.title,
            preview: value.preview,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
