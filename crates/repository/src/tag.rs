use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_core::{
    Tag,
    tag::{TagId, TagRepository},
};
use colette_handler::{TagDto, TagQueryParams, TagQueryRepository};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagRepository for PostgresTagRepository {
    async fn find_by_id(&self, id: TagId, user_id: UserId) -> Result<Option<Tag>, RepositoryError> {
        let tag = sqlx::query_file_as!(
            TagByIdRow,
            "queries/tags/find_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tag)
    }

    async fn save(&self, data: &Tag) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/tags/upsert.sql",
            data.id().as_inner(),
            data.title().as_inner(),
            data.user_id().as_inner(),
            data.created_at(),
            data.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => RepositoryError::Duplicate,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: TagId, user_id: UserId) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/tags/delete_by_id.sql",
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

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TagByIdRow {
    id: Uuid,
    title: String,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TagByIdRow> for Tag {
    fn from(value: TagByIdRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.title,
            value.user_id,
            value.created_at,
            value.updated_at,
        )
    }
}

#[async_trait::async_trait]
impl TagQueryRepository for PostgresTagRepository {
    async fn query(&self, params: TagQueryParams) -> Result<Vec<TagDto>, RepositoryError> {
        let tags = sqlx::query_file_as!(
            TagRow,
            "queries/tags/find.sql",
            params.user_id,
            params.id,
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TagRow {
    id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TagRow> for TagDto {
    fn from(value: TagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
