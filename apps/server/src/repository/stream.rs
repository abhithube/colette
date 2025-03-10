use colette_core::{
    Stream,
    common::Transaction,
    stream::{
        Error, StreamById, StreamCreateParams, StreamDeleteParams, StreamFindByIdParams,
        StreamFindParams, StreamRepository, StreamUpdateParams,
    },
};
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};

use super::common::parse_timestamp;

#[derive(Debug, Clone)]
pub struct SqliteStreamRepository {
    pool: Pool<Sqlite>,
}

impl SqliteStreamRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StreamRepository for SqliteStreamRepository {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, StreamRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_stream_by_id(
        &self,
        tx: &dyn Transaction,
        params: StreamFindByIdParams,
    ) -> Result<StreamById, Error> {
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

        Ok(StreamById {
            id: row.get::<String, _>(0).parse().unwrap(),
            user_id: row.get::<String, _>(1).parse().unwrap(),
        })
    }

    async fn create_stream(&self, params: StreamCreateParams) -> Result<(), Error> {
        let title = params.title.clone();

        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(title),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamUpdateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        if params.title.is_none() && params.filter.is_none() {
            return Ok(());
        }

        let (sql, values) = params.into_update().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }

    async fn delete_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamDeleteParams,
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
}

#[derive(sqlx::FromRow)]
struct StreamRow {
    id: String,
    title: String,
    filter_raw: String,
    user_id: String,
    created_at: i32,
    updated_at: i32,
}

impl From<StreamRow> for Stream {
    fn from(value: StreamRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
