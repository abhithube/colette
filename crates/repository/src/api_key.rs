use chrono::{DateTime, Utc};
use colette_core::{
    ApiKey, RepositoryError,
    api_key::{
        ApiKeyById, ApiKeyByLookupHash, ApiKeyFindParams, ApiKeyInsertParams, ApiKeyRepository,
        ApiKeyUpdateParams,
    },
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
            params.id,
            params.user_id,
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKeyById>, RepositoryError> {
        let api_key = sqlx::query_file_as!(ApiKeyByIdRow, "queries/api_keys/find_by_id.sql", id)
            .map(Into::into)
            .fetch_optional(&self.pool)
            .await?;

        Ok(api_key)
    }

    async fn find_by_lookup_hash(
        &self,
        lookup_hash: String,
    ) -> Result<Option<ApiKeyByLookupHash>, RepositoryError> {
        let api_key = sqlx::query_file_as!(
            ApiKeyByLookupHashRow,
            "queries/api_keys/find_by_lookup_hash.sql",
            lookup_hash
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    async fn insert(&self, params: ApiKeyInsertParams) -> Result<ApiKey, RepositoryError> {
        let api_key = sqlx::query_file_as!(
            ApiKeyRow,
            "queries/api_keys/insert.sql",
            params.lookup_hash,
            params.verification_hash,
            params.title,
            params.preview,
            params.user_id
        )
        .map(Into::into)
        .fetch_one(&self.pool)
        .await?;

        Ok(api_key)
    }

    async fn update(&self, params: ApiKeyUpdateParams) -> Result<(), RepositoryError> {
        sqlx::query_file!("queries/api_keys/update.sql", params.id, params.title)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query_file!("queries/api_keys/delete_by_id.sql", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

struct ApiKeyRow {
    id: Uuid,
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
            title: value.title,
            preview: value.preview,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

struct ApiKeyByIdRow {
    id: Uuid,
    user_id: Uuid,
}

impl From<ApiKeyByIdRow> for ApiKeyById {
    fn from(value: ApiKeyByIdRow) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
        }
    }
}

struct ApiKeyByLookupHashRow {
    id: Uuid,
    verification_hash: String,
    user_id: Uuid,
}

impl From<ApiKeyByLookupHashRow> for ApiKeyByLookupHash {
    fn from(value: ApiKeyByLookupHashRow) -> Self {
        Self {
            id: value.id,
            verification_hash: value.verification_hash,
            user_id: value.user_id,
        }
    }
}
