use chrono::{DateTime, Utc};
use colette_core::{
    Collection,
    bookmark::BookmarkFilter,
    collection::{CollectionParams, CollectionRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    collection::{CollectionDelete, CollectionInsert, CollectionSelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresCollectionRepository {
    pool: PgPool,
}

impl PostgresCollectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for PostgresCollectionRepository {
    async fn query(&self, params: CollectionParams) -> Result<Vec<Collection>, Error> {
        let (sql, values) = CollectionSelect {
            id: params.id,
            user_id: params.user_id,
            cursor: params.cursor.as_deref(),
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, CollectionRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Collection) -> Result<(), Error> {
        let (sql, values) = CollectionInsert {
            id: data.id,
            title: &data.title,
            filter: serde_json::to_value(&data.filter).unwrap(),
            user_id: data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
        .into_insert()
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(data.title.clone())
                }
                _ => Error::Sqlx(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = CollectionDelete { id }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CollectionRow {
    id: Uuid,
    title: String,
    filter_json: Json<BookmarkFilter>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionRow> for Collection {
    fn from(value: CollectionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter_json.0,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
