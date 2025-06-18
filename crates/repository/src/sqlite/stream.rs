use colette_core::{
    Stream,
    stream::{Error, StreamParams, StreamRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    stream::{StreamDelete, StreamInsert},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteStreamRepository {
    pool: Pool,
}

impl SqliteStreamRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StreamRepository for SqliteStreamRepository {
    async fn query(&self, params: StreamParams) -> Result<Vec<Stream>, Error> {
        let client = self.pool.get().await?;

        let streams = client
            .interact(move |conn| {
                let (sql, values) = params.into_select().build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Stream>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(streams)
    }

    async fn save(&self, data: &Stream) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let (sql, values) = StreamInsert {
                    id: data.id,
                    title: &data.title,
                    filter: serde_json::to_value(&data.filter).unwrap(),
                    user_id: data.user_id,
                    created_at: data.created_at,
                    updated_at: data.updated_at,
                }
                .into_insert()
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values).map_err(|e| {
                    match e.sqlite_error().map(|e| e.extended_code) {
                        Some(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) => {
                            Error::Conflict(data.title.clone())
                        }
                        _ => Error::SqliteClient(e),
                    }
                })
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = StreamDelete { id }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Stream {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            title: value.get_unwrap("title"),
            filter: serde_json::from_value(value.get_unwrap("filter_json")).unwrap(),
            user_id: value.get_unwrap("user_id"),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
        }
    }
}
