use chrono::{DateTime, Utc};
use colette_core::{
    Collection,
    collection::{CollectionFindParams, CollectionRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    collection::{CollectionDelete, CollectionInsert, CollectionSelect, CollectionSelectOne},
};
use libsql::{Connection, ffi::SQLITE_CONSTRAINT_UNIQUE};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlCollectionRepository {
    conn: Connection,
}

impl LibsqlCollectionRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for LibsqlCollectionRepository {
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, Error> {
        let (sql, values) = CollectionSelect {
            id: params.id,
            user_id: params.user_id.as_deref(),
            cursor: params.cursor.as_deref(),
            limit: params.limit,
        }
        .into_select()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut collections = Vec::<Collection>::new();
        while let Some(row) = rows.next().await? {
            collections.push(libsql::de::from_row::<CollectionRow>(&row)?.into());
        }

        Ok(collections)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Collection>, Error> {
        let (sql, values) = CollectionSelectOne { id }
            .into_select()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row::<CollectionRow>(&row)?.into()))
    }

    async fn save(&self, data: &Collection, upsert: bool) -> Result<(), Error> {
        let (sql, values) = CollectionInsert {
            id: data.id,
            title: &data.title,
            filter_raw: &serde_json::to_string(&data.filter).unwrap(),
            user_id: &data.user_id,
            upsert,
        }
        .into_insert()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params())
            .await
            .map_err(|e| match e {
                libsql::Error::SqliteFailure(SQLITE_CONSTRAINT_UNIQUE, _) => {
                    Error::Conflict(data.title.clone())
                }
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = CollectionDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct CollectionRow {
    id: Uuid,
    title: String,
    filter_raw: String,
    user_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionRow> for Collection {
    fn from(value: CollectionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
