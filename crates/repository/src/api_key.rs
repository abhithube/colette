use chrono::{DateTime, Utc};
use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyFindParams, ApiKeyId, ApiKeyInsertParams, ApiKeyRepository, ApiKeyUpdateParams,
    },
    common::RepositoryError,
};
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
    async fn find(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, RepositoryError> {
        let api_keys = sqlx::query_file_as!(
            ApiKeyRow,
            "queries/api_keys/find.sql",
            params.id.map(|e| e.as_inner()),
            params.lookup_hash,
            params.user_id.map(|e| e.as_inner()),
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    async fn insert(&self, params: ApiKeyInsertParams) -> Result<ApiKey, RepositoryError> {
        let api_key = sqlx::query_file_as!(
            ApiKeyRow,
            "queries/api_keys/insert.sql",
            params.lookup_hash,
            params.verification_hash,
            params.title,
            params.preview,
            params.user_id.as_inner()
        )
        .map(Into::into)
        .fetch_one(&self.pool)
        .await?;

        Ok(api_key)
    }

    async fn update(&self, params: ApiKeyUpdateParams) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/api_keys/update.sql",
            params.id.as_inner(),
            params.title
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: ApiKeyId) -> Result<(), RepositoryError> {
        sqlx::query_file!("queries/api_keys/delete_by_id.sql", id.as_inner())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

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
            id: value.id.into(),
            lookup_hash: value.lookup_hash,
            verification_hash: value.verification_hash,
            title: value.title,
            preview: value.preview,
            user_id: value.user_id.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
