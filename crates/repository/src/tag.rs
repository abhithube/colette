use chrono::{DateTime, Utc};
use colette_core::{
    Tag,
    common::RepositoryError,
    tag::{TagFindParams, TagId, TagInsertParams, TagRepository, TagUpdateParams},
};
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
    async fn find(&self, params: TagFindParams) -> Result<Vec<Tag>, RepositoryError> {
        let tags = sqlx::query_file_as!(
            TagRow,
            "queries/tags/find.sql",
            params.id.map(|e| e.as_inner()),
            params.user_id.map(|e| e.as_inner()),
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    async fn insert(&self, params: TagInsertParams) -> Result<TagId, RepositoryError> {
        let id = sqlx::query_file_scalar!(
            "queries/tags/insert.sql",
            params.title,
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

    async fn update(&self, params: TagUpdateParams) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/tags/update.sql",
            params.id.as_inner(),
            params.title
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: TagId) -> Result<(), RepositoryError> {
        sqlx::query_file!("queries/tags/delete_by_id.sql", id.as_inner())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

pub(crate) struct TagRow {
    pub(crate) id: Uuid,
    pub(crate) title: String,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TagRow> for Tag {
    fn from(value: TagRow) -> Self {
        Self {
            id: value.id.into(),
            title: value.title,
            user_id: value.user_id.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
