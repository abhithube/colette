use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{
        BookmarkById, BookmarkCreateParams, BookmarkDeleteParams, BookmarkFindByIdParams,
        BookmarkFindParams, BookmarkRepository, BookmarkTagsLinkParams, BookmarkUpdateParams,
        BookmarkUpsertParams, Error,
    },
    common::Transaction,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark_tag::{BookmarkTagDeleteMany, BookmarkTagSelectMany, BookmarkTagUpsertMany},
};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};

#[derive(Debug, Clone)]
pub struct SqliteBookmarkRepository {
    pool: Pool<Sqlite>,
}

impl SqliteBookmarkRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn find_bookmarks(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let bookmark_rows = sqlx::query_as_with::<_, BookmarkRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let (sql, values) = BookmarkTagSelectMany {
            bookmark_ids: bookmark_rows.iter().map(|e| e.id.to_string()),
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let tag_rows = sqlx::query_as_with::<_, BookmarkTagRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let mut tag_row_map = HashMap::<String, Vec<BookmarkTagRow>>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.bookmark_id.clone())
                .or_default()
                .push(row);
        }

        let bookmarks = bookmark_rows
            .into_iter()
            .map(|bookmark| {
                BookmarkRowWithTagRows {
                    tags: tag_row_map.remove(&bookmark.id),
                    bookmark,
                }
                .into()
            })
            .collect();

        Ok(bookmarks)
    }

    async fn find_bookmark_by_id(
        &self,
        tx: &dyn Transaction,
        params: BookmarkFindByIdParams,
    ) -> Result<BookmarkById, Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let id = params.id;

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_with(&sql, values)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Database(e),
            })?;

        Ok(BookmarkById {
            id: row.get::<String, _>(0).parse().unwrap(),
            user_id: row.get::<String, _>(1).parse().unwrap(),
        })
    }

    async fn create_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkCreateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let url = params.url.clone();

        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(tx.as_mut())
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(url),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_bookmark(
        &self,
        tx: Option<&dyn Transaction>,
        params: BookmarkUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.map(|e| {
            e.as_any()
                .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
                .unwrap()
        });

        if params.title.is_none()
            && params.thumbnail_url.is_none()
            && params.published_at.is_none()
            && params.author.is_none()
            && params.archived_path.is_none()
        {
            return Ok(());
        }

        let (sql, values) = params.into_update().build_sqlx(SqliteQueryBuilder);

        if let Some(tx) = tx {
            let mut tx = tx.lock().await;

            sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;
        } else {
            sqlx::query_with(&sql, values).execute(&self.pool).await?;
        }

        Ok(())
    }

    async fn delete_bookmark(
        &self,
        tx: &dyn Transaction,
        params: BookmarkDeleteParams,
    ) -> Result<(), Error> {
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

    async fn upsert(&self, params: BookmarkUpsertParams) -> Result<(), Error> {
        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn link_tags(
        &self,
        tx: &dyn Transaction,
        params: BookmarkTagsLinkParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;
        let conn = tx.as_mut();

        let (sql, values) = BookmarkTagDeleteMany {
            bookmark_id: params.bookmark_id,
            tag_ids: params.tags.iter().map(|e| e.id.to_string()),
        }
        .into_delete()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;

        let (sql, values) = BookmarkTagUpsertMany {
            bookmark_id: params.bookmark_id,
            tags: params.tags,
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(conn).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct BookmarkRow {
    pub id: String,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub archived_path: Option<String>,
    pub author: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct BookmarkTagRow {
    pub bookmark_id: String,
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BookmarkTagRow> for Tag {
    fn from(value: BookmarkTagRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: value.created_at,
            updated_at: value.updated_at,
            ..Default::default()
        }
    }
}

pub struct BookmarkRowWithTagRows {
    pub bookmark: BookmarkRow,
    pub tags: Option<Vec<BookmarkTagRow>>,
}

impl From<BookmarkRowWithTagRows> for Bookmark {
    fn from(value: BookmarkRowWithTagRows) -> Self {
        Self {
            id: value.bookmark.id.parse().unwrap(),
            link: value.bookmark.link.parse().unwrap(),
            title: value.bookmark.title,
            thumbnail_url: value.bookmark.thumbnail_url.and_then(|e| e.parse().ok()),
            published_at: value.bookmark.published_at,
            author: value.bookmark.author,
            archived_path: value.bookmark.archived_path,
            user_id: value.bookmark.user_id.parse().unwrap(),
            created_at: value.bookmark.created_at,
            updated_at: value.bookmark.updated_at,
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}
