use chrono::DateTime;
use colette_core::{
    Stream,
    stream::{Error, StreamParams, StreamRepository},
};
use colette_query::{
    IntoInsert, IntoDelete, IntoSelect,
    stream::{StreamDelete, StreamInsert},
};
use libsql::{Connection, ffi::SQLITE_CONSTRAINT_UNIQUE};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlStreamRepository {
    conn: Connection,
}

impl LibsqlStreamRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl StreamRepository for LibsqlStreamRepository {
    async fn query(&self, params: StreamParams) -> Result<Vec<Stream>, Error> {
        let (sql, values) = params.into_select().build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut streams = Vec::<Stream>::new();
        while let Some(row) = rows.next().await? {
            streams.push(libsql::de::from_row::<StreamRow>(&row)?.into());
        }

        Ok(streams)
    }

    async fn save(&self, data: &Stream) -> Result<(), Error> {
        let (sql, values) = StreamInsert {
            id: data.id,
            title: &data.title,
            filter_raw: &serde_json::to_string(&data.filter).unwrap(),
            user_id: &data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
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
        let (sql, values) = StreamDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct StreamRow {
    id: Uuid,
    title: String,
    filter_raw: String,
    user_id: String,
    created_at: i64,
    updated_at: i64,
}

impl From<StreamRow> for Stream {
    fn from(value: StreamRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            user_id: value.user_id,
            created_at: DateTime::from_timestamp(value.created_at, 0).unwrap(),
            updated_at: DateTime::from_timestamp(value.updated_at, 0).unwrap(),
        }
    }
}
