use colette_core::{
    Stream,
    stream::{Error, StreamParams, StreamRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    stream::{StreamDelete, StreamInsert},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use tokio_postgres::error::SqlState;
use uuid::Uuid;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresStreamRepository {
    pool: Pool,
}

impl PostgresStreamRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StreamRepository for PostgresStreamRepository {
    async fn query(&self, params: StreamParams) -> Result<Vec<Stream>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);
        let streams = client.query_prepared::<Stream>(&sql, &values).await?;

        Ok(streams)
    }

    async fn save(&self, data: &Stream) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = StreamInsert {
            id: data.id,
            title: &data.title,
            filter: serde_json::to_value(&data.filter).unwrap(),
            user_id: data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
        .into_insert()
        .build_postgres(PostgresQueryBuilder);

        client
            .execute_prepared(&sql, &values)
            .await
            .map_err(|e| match e.code() {
                Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title.clone()),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = StreamDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Stream {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            title: value.get("title"),
            filter: serde_json::from_value(value.get("filter_json")).unwrap(),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}
