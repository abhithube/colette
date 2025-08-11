use chrono::{DateTime, Utc};
use colette_core::{
    Collection, RepositoryError,
    bookmark::BookmarkFilter,
    collection::{
        CollectionFindParams, CollectionId, CollectionInsertParams, CollectionRepository,
        CollectionUpdateParams,
    },
};
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
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, RepositoryError> {
        let collections = sqlx::query_file_as!(
            CollectionRow,
            "queries/collections/find.sql",
            params.id.map(|e| e.as_inner()),
            params.user_id.map(|e| e.as_inner()),
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(collections)
    }

    async fn insert(
        &self,
        params: CollectionInsertParams,
    ) -> Result<CollectionId, RepositoryError> {
        let id = sqlx::query_file_scalar!(
            "queries/collections/insert.sql",
            params.title,
            Json(params.filter) as Json<BookmarkFilter>,
            params.user_id.as_inner()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => RepositoryError::Duplicate,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(id.into())
    }

    async fn update(&self, params: CollectionUpdateParams) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/collections/update.sql",
            params.id.as_inner(),
            params.title,
            params.filter.map(Json) as Option<Json<BookmarkFilter>>
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: CollectionId) -> Result<(), RepositoryError> {
        sqlx::query_file!("queries/collections/delete_by_id.sql", id.as_inner())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

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
            id: value.id.into(),
            title: value.title,
            filter: value.filter_json.0,
            user_id: value.user_id.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
