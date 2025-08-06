use chrono::{DateTime, Utc};
use colette_core::{
    Tag,
    tag::{Error, TagById, TagFindParams, TagInsertParams, TagRepository, TagUpdateParams},
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
    async fn find(&self, params: TagFindParams) -> Result<Vec<Tag>, Error> {
        let tags = sqlx::query_file_as!(
            TagRow,
            "queries/tags/find.sql",
            params.id,
            params.user_id,
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<TagById>, Error> {
        let tag = sqlx::query_file_as!(TagByIdRow, "queries/tags/find_by_id.sql", id)
            .map(Into::into)
            .fetch_optional(&self.pool)
            .await?;

        Ok(tag)
    }

    async fn insert(&self, params: TagInsertParams) -> Result<Uuid, Error> {
        let id = sqlx::query_file_scalar!("queries/tags/insert.sql", params.title, params.user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(params.title)
                }
                _ => Error::Sqlx(e),
            })?;

        Ok(id)
    }

    async fn update(&self, params: TagUpdateParams) -> Result<(), Error> {
        sqlx::query_file!("queries/tags/update.sql", params.id, params.title)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query_file!("queries/tags/delete_by_id.sql", id)
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
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

struct TagByIdRow {
    id: Uuid,
    user_id: Uuid,
}

impl From<TagByIdRow> for TagById {
    fn from(value: TagByIdRow) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
        }
    }
}
