use colette_core::{
    Collection,
    collection::{CollectionParams, CollectionRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    collection::{CollectionDelete, CollectionInsert, CollectionSelect},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    pool: Pool,
}

impl SqliteCollectionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn query(&self, params: CollectionParams) -> Result<Vec<Collection>, Error> {
        let client = self.pool.get().await?;

        let collections = client
            .interact(move |conn| {
                let (sql, values) = CollectionSelect {
                    id: params.id,
                    user_id: params.user_id,
                    cursor: params.cursor.as_deref(),
                    limit: params.limit,
                }
                .into_select()
                .build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Collection>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(collections)
    }

    async fn save(&self, data: &Collection) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let (sql, values) = CollectionInsert {
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
                let (sql, values) = CollectionDelete { id }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Collection {
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
