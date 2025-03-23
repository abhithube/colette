use chrono::{DateTime, Utc};
use colette_core::{
    ApiKey,
    api_key::{ApiKeyFindOne, ApiKeyFindParams, ApiKeyRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    api_key::{ApiKeyDelete, ApiKeyInsert, ApiKeySelect, ApiKeySelectOne},
};
use libsql::Connection;
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlApiKeyRepository {
    conn: Connection,
}

impl LibsqlApiKeyRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl ApiKeyRepository for LibsqlApiKeyRepository {
    async fn find(&self, params: ApiKeyFindParams) -> Result<Vec<ApiKey>, Error> {
        let (sql, values) = ApiKeySelect {
            id: params.id,
            user_id: params.user_id.as_deref(),
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut api_keys = Vec::<ApiKey>::new();
        while let Some(row) = rows.next().await? {
            api_keys.push(libsql::de::from_row::<ApiKeyRow>(&row)?.into());
        }

        Ok(api_keys)
    }

    async fn find_one(&self, key: ApiKeyFindOne) -> Result<Option<ApiKey>, Error> {
        let key = match key {
            ApiKeyFindOne::Id(id) => ApiKeySelectOne::Id(id),
            ApiKeyFindOne::LookupHash(lookup_hash) => ApiKeySelectOne::LookupHash(lookup_hash),
        };

        let (sql, values) = key.into_select().build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row::<ApiKeyRow>(&row)?.into()))
    }

    async fn save(&self, data: &ApiKey, upsert: bool) -> Result<(), Error> {
        let (sql, values) = ApiKeyInsert {
            id: data.id,
            lookup_hash: &data.lookup_hash,
            verification_hash: &data.verification_hash,
            title: &data.title,
            preview: &data.preview,
            user_id: &data.user_id,
            upsert,
        }
        .into_insert()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = ApiKeyDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct ApiKeyRow {
    id: Uuid,
    lookup_hash: String,
    verification_hash: String,
    title: String,
    preview: String,
    user_id: String,
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
