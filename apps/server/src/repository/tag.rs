use colette_core::{
    Tag,
    common::Transaction,
    tag::{
        Error, TagById, TagCreateParams, TagDeleteParams, TagFindByIdsParams, TagFindParams,
        TagRepository, TagUpdateParams,
    },
};
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};

use super::common::parse_timestamp;

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
    async fn find_tags(&self, params: TagFindParams) -> Result<Vec<Tag>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, TagRowWithCounts, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_tags_by_ids(
        &self,
        tx: &dyn Transaction,
        params: TagFindByIdsParams,
    ) -> Result<Vec<TagById>, Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_with(&sql, values)
            .fetch_all(tx.as_mut())
            .await?;

        Ok(rows
            .into_iter()
            .map(|e| TagById {
                id: e.get::<String, _>(0).parse().unwrap(),
                user_id: e.get::<String, _>(1).parse().unwrap(),
            })
            .collect())
    }

    async fn create_tag(&self, params: TagCreateParams) -> Result<(), Error> {
        let title = params.title.clone();

        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(title),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_tag(&self, tx: &dyn Transaction, params: TagUpdateParams) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        if params.title.is_none() {
            return Ok(());
        }

        let (sql, values) = params.into_update().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }

    async fn delete_tag(&self, tx: &dyn Transaction, params: TagDeleteParams) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let (sql, values) = params.into_delete().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct TagRowWithCounts {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
    pub feed_count: i64,
    pub bookmark_count: i64,
}

impl From<TagRowWithCounts> for Tag {
    fn from(value: TagRowWithCounts) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            feed_count: Some(value.feed_count),
            bookmark_count: Some(value.bookmark_count),
        }
    }
}
