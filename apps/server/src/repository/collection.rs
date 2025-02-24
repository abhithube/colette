use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Collection,
    collection::{
        CollectionBookmarkFindParams, CollectionCreateData, CollectionFindParams,
        CollectionRepository, CollectionUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
};
use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder, types::Json};
use uuid::Uuid;

use crate::repository::{bookmark::BookmarkRow, common::ToSql};

#[derive(Debug, Clone)]
pub struct PostgresCollectionRepository {
    pool: Pool<Postgres>,
}

impl PostgresCollectionRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresCollectionRepository {
    type Params = CollectionFindParams;
    type Output = Result<Vec<Collection>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let collections = sqlx::query_file_as!(
            CollectionRow,
            "queries/collections/select.sql",
            params.user_id,
            params.id.is_none(),
            params.id,
            params.cursor.is_none(),
            params.cursor.map(|e| e.title),
            params.limit
        )
        .fetch_all(&self.pool)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(collections)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresCollectionRepository {
    type Data = CollectionCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        sqlx::query_file_scalar!(
            "queries/collections/insert.sql",
            data.title,
            serde_json::to_value(data.filter).unwrap(),
            data.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Database(e),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresCollectionRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            sqlx::query_file!(
                "queries/collections/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title,
                data.filter.is_some(),
                data.filter.map(|e| serde_json::to_value(e).unwrap())
            )
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresCollectionRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = sqlx::query_file!("queries/collections/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl CollectionRepository for PostgresCollectionRepository {
    async fn find_bookmarks(
        &self,
        params: CollectionBookmarkFindParams,
    ) -> Result<Vec<Bookmark>, Error> {
        let initial = format!(
            r#"WITH json_tags AS (
  SELECT bt.bookmark_id,
         jsonb_agg(jsonb_build_object('id', t.id, 'title', t.title) ORDER BY t.title) AS tags
    FROM bookmark_tags bt
    JOIN tags t ON t.id = bt.tag_id
   WHERE bt.user_id = '{0}'
   GROUP BY bt.bookmark_id
)
SELECT b.id,
       b.link,
       b.title,
       b.thumbnail_url,
       b.published_at,
       b.author,
       b.archived_path,
       b.created_at,
       b.updated_at,
       coalesce(jt.tags, '[]'::jsonb) AS tags
  FROM bookmarks b
  LEFT JOIN json_tags jt ON jt.bookmark_id = b.id
 WHERE b.user_id = '{0}'"#,
            params.user_id
        );

        let mut qb = QueryBuilder::new(initial);

        let where_clause = params.filter.to_sql();
        if !where_clause.is_empty() {
            qb.push(" AND ");
            qb.push(&where_clause);
        }

        if let Some(cursor) = params.cursor {
            qb.push(" AND b.created_at > ");
            qb.push_bind(cursor.created_at);
        }

        qb.push("\n ORDER BY b.created_at ASC");

        if let Some(limit) = params.limit {
            qb.push("\n LIMIT ");
            qb.push_bind(limit);
        }

        let query = qb.build_query_as::<BookmarkRow>();

        let bookmarks = query
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(bookmarks)
    }
}

struct CollectionRow {
    id: Uuid,
    title: String,
    filter: Json<Value>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<CollectionRow> for Collection {
    fn from(value: CollectionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: serde_json::from_value(value.filter.0).unwrap(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
