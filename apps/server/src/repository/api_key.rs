use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyCreateData, ApiKeyFindParams, ApiKeyRepository, ApiKeySearchParams, ApiKeySearched,
        ApiKeyUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresApiKeyRepository {
    pool: Pool<Postgres>,
}

impl PostgresApiKeyRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresApiKeyRepository {
    type Params = ApiKeyFindParams;
    type Output = Result<Vec<ApiKey>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let api_keys = sqlx::query_file_as!(
            ApiKey,
            "queries/api_keys/select.sql",
            params.user_id,
            params.id.is_none(),
            params.id,
            params.cursor.is_none(),
            params.cursor.map(|e| e.created_at),
            params.limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresApiKeyRepository {
    type Data = ApiKeyCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        sqlx::query_file_scalar!(
            "queries/api_keys/insert.sql",
            data.lookup_hash,
            data.verification_hash,
            data.title,
            data.preview,
            data.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Database(e),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresApiKeyRepository {
    type Params = IdParams;
    type Data = ApiKeyUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            sqlx::query_file!(
                "queries/api_keys/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title
            )
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresApiKeyRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = sqlx::query_file!("queries/api_keys/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn search(&self, params: ApiKeySearchParams) -> Result<Option<ApiKeySearched>, Error> {
        let api_key = sqlx::query_file_as!(
            ApiKeySearched,
            "queries/api_keys/search.sql",
            params.lookup_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }
}
