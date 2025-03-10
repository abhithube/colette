use colette_core::{
    ApiKey,
    api_key::{
        ApiKeyById, ApiKeyCreateParams, ApiKeyDeleteParams, ApiKeyFindByIdParams, ApiKeyFindParams,
        ApiKeyRepository, ApiKeySearchParams, ApiKeySearched, ApiKeyUpdateParams, Error,
    },
    common::Transaction,
};
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};

use super::common::parse_timestamp;

#[derive(Debug, Clone)]
pub struct SqliteApiKeyRepository {
    pool: Pool<Sqlite>,
}

impl SqliteApiKeyRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for SqliteApiKeyRepository {
    async fn find_api_keys(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, ApiKeyRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_api_key_by_id(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyFindByIdParams,
    ) -> Result<ApiKeyById, Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let id = params.id;

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_with(&sql, values)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Database(e),
            })?;

        Ok(ApiKeyById {
            id: row.get::<String, _>(0).parse().unwrap(),
            user_id: row.get::<String, _>(1).parse().unwrap(),
        })
    }

    async fn create_api_key(&self, params: ApiKeyCreateParams) -> Result<(), Error> {
        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn update_api_key(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyUpdateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        if params.title.is_none() {
            return Ok(());
        }

        let (sql, values) = params.into_update().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }

    async fn delete_api_key(
        &self,
        tx: &dyn Transaction,
        params: ApiKeyDeleteParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let (sql, values) = params.into_delete().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }

    async fn search_api_key(
        &self,
        params: ApiKeySearchParams,
    ) -> Result<Option<ApiKeySearched>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_with(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|e| ApiKeySearched {
            verification_hash: e.get::<String, _>(0),
            user_id: e.get::<String, _>(1).parse().unwrap(),
        }))
    }
}

#[derive(sqlx::FromRow)]
struct ApiKeyRow {
    id: String,
    title: String,
    preview: String,
    user_id: String,
    created_at: i32,
    updated_at: i32,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(value: ApiKeyRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            preview: value.preview,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
