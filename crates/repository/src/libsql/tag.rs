use chrono::{DateTime, Utc};
use colette_core::{
    Tag,
    tag::{Error, TagParams, TagRepository},
};
use colette_query::{
    IntoInsert, IntoDelete, IntoSelect,
    tag::{TagDelete, TagInsert},
};
use libsql::{Connection, ffi::SQLITE_CONSTRAINT_UNIQUE};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlTagRepository {
    conn: Connection,
}

impl LibsqlTagRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TagRepository for LibsqlTagRepository {
    async fn query(&self, params: TagParams) -> Result<Vec<Tag>, Error> {
        let (sql, values) = params.into_select().build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut tags = Vec::<Tag>::new();
        while let Some(row) = rows.next().await? {
            tags.push(libsql::de::from_row::<TagRow>(&row)?.into());
        }

        Ok(tags)
    }

    async fn save(&self, data: &Tag) -> Result<(), Error> {
        let (sql, values) = TagInsert {
            id: data.id,
            title: &data.title,
            user_id: &data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
            upsert: false,
        }
        .into_insert()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params())
            .await
            .map_err(|e| match e {
                libsql::Error::SqliteFailure(SQLITE_CONSTRAINT_UNIQUE, _) => {
                    Error::Conflict(data.title.clone())
                }
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = TagDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
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
