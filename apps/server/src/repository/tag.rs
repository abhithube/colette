use chrono::{DateTime, Utc};
use colette_core::{
    Tag,
    tag::{Error, TagFindParams, TagRepository, TagUpsertType},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    tag::{TagDelete, TagInsert, TagSelect, TagSelectOne},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteTagRepository {
    pool: Pool<Sqlite>,
}

impl SqliteTagRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagRepository for SqliteTagRepository {
    async fn find(&self, params: TagFindParams) -> Result<Vec<Tag>, Error> {
        let (sql, values) = TagSelect {
            ids: params.ids,
            tag_type: params.tag_type,
            feed_id: params.feed_id,
            bookmark_id: params.bookmark_id,
            user_id: params.user_id.as_deref(),
            cursor: params.cursor.as_deref(),
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, TagRowWithCounts, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Tag>, Error> {
        let (sql, values) = TagSelectOne::Ids(ids)
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, TagRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Tag, upsert: Option<TagUpsertType>) -> Result<(), Error> {
        let (sql, values) = TagInsert {
            id: data.id,
            title: &data.title,
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
        let (sql, values) = TagDelete { id }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct TagRow {
    pub id: Uuid,
    pub title: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TagRow> for Tag {
    fn from(value: TagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            feed_count: None,
            bookmark_count: None,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct TagRowWithCounts {
    pub id: Uuid,
    pub title: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub feed_count: i64,
    pub bookmark_count: i64,
}

impl From<TagRowWithCounts> for Tag {
    fn from(value: TagRowWithCounts) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            feed_count: Some(value.feed_count),
            bookmark_count: Some(value.bookmark_count),
        }
    }
}
