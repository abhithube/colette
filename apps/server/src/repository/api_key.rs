use chrono::{DateTime, Utc};
use colette_core::{
    ApiKey,
    api_key::{ApiKeyFindOne, ApiKeyFindParams, ApiKeyRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    api_key::{ApiKeyDelete, ApiKeyInsert, ApiKeySelect, ApiKeySelectOne},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

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
    async fn find(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error> {
        let (sql, values) = ApiKeySelect {
            id: params.id,
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, ApiKeyRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_one(&self, key: ApiKeyFindOne) -> Result<Option<ApiKey>, Error> {
        let key = match key {
            ApiKeyFindOne::Id(id) => ApiKeySelectOne::Id(id),
            ApiKeyFindOne::LookupHash(lookup_hash) => ApiKeySelectOne::LookupHash(lookup_hash),
        };

        let (sql, values) = key.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, ApiKeyRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, data: &ApiKey, upsert: bool) -> Result<(), Error> {
        let (sql, values) = ApiKeyInsert {
            id: data.id,
            lookup_hash: &data.lookup_hash,
            verification_hash: &data.verification_hash,
            title: &data.title,
            preview: &data.preview,
            user_id: data.user_id,
            upsert,
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = ApiKeyDelete { id }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
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
