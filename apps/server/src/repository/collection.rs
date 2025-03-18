use chrono::{DateTime, Utc};
use colette_core::{
    Collection,
    collection::{CollectionFindParams, CollectionRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    collection::{CollectionDelete, CollectionInsert, CollectionSelect, CollectionSelectOne},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    pool: Pool<Sqlite>,
}

impl SqliteCollectionRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, Error> {
        let (sql, values) = CollectionSelect {
            id: params.id,
            user_id: params.user_id.as_deref(),
            cursor: params.cursor.as_deref(),
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, CollectionRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Collection>, Error> {
        let (sql, values) = CollectionSelectOne { id }
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, CollectionRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
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
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(data.title.clone())
                }
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = CollectionDelete { id }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
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
