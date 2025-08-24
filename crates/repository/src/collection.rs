use chrono::{DateTime, Utc};
use colette_core::{
    Collection,
    auth::UserId,
    bookmark::BookmarkFilter,
    collection::{CollectionId, CollectionRepository},
    common::RepositoryError,
};
use colette_handler::{CollectionDto, CollectionQueryParams, CollectionQueryRepository};
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
    async fn find_by_id(
        &self,
        id: CollectionId,
        user_id: UserId,
    ) -> Result<Option<Collection>, RepositoryError> {
        let collection = sqlx::query_file_as!(
            CollectionByIdRow,
            "queries/collections/find_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(collection)
    }

    async fn save(&self, data: &Collection) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/collections/upsert.sql",
            data.id().as_inner(),
            data.title().as_inner(),
            Json(data.filter().to_owned()) as Json<BookmarkFilter>,
            data.user_id().as_inner(),
            data.created_at(),
            data.updated_at()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => RepositoryError::Duplicate,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: CollectionId, user_id: UserId) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/collections/delete_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(())
    }
}

struct CollectionByIdRow {
    id: Uuid,
    title: String,
    filter_json: Json<BookmarkFilter>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionByIdRow> for Collection {
    fn from(value: CollectionByIdRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.title,
            value.filter_json.0,
            value.user_id,
            value.created_at,
            value.updated_at,
        )
    }
}

#[async_trait::async_trait]
impl CollectionQueryRepository for PostgresCollectionRepository {
    async fn query(
        &self,
        params: CollectionQueryParams,
    ) -> Result<Vec<CollectionDto>, RepositoryError> {
        let collections = sqlx::query_file_as!(
            CollectionRow,
            "queries/collections/find.sql",
            params.user_id,
            params.id,
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(collections)
    }
}

struct CollectionRow {
    id: Uuid,
    title: String,
    filter_json: Json<BookmarkFilter>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionRow> for CollectionDto {
    fn from(value: CollectionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter_json.0,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
