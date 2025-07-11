use colette_core::{
    Collection,
    collection::{CollectionParams, CollectionRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    collection::{CollectionDelete, CollectionInsert, CollectionSelect},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use tokio_postgres::error::SqlState;
use uuid::Uuid;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresCollectionRepository {
    pool: Pool,
}

impl PostgresCollectionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for PostgresCollectionRepository {
    async fn query(&self, params: CollectionParams) -> Result<Vec<Collection>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = CollectionSelect {
            id: params.id,
            user_id: params.user_id,
            cursor: params.cursor.as_deref(),
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let collections = client.query_prepared::<Collection>(&sql, &values).await?;

        Ok(collections)
    }

    async fn save(&self, data: &Collection) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = CollectionInsert {
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
                _ => Error::PostgresClient(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = CollectionDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Collection {
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
