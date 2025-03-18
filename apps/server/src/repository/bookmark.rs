use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{BookmarkFindParams, BookmarkRepository, BookmarkUpsertType, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    bookmark::{BookmarkDelete, BookmarkInsert, BookmarkSelect, BookmarkSelectOne},
    bookmark_tag::{BookmarkTagById, BookmarkTagDelete, BookmarkTagInsert, BookmarkTagSelect},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

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
    async fn find(&self, params: BookmarkFindParams) -> Result<Vec<Bookmark>, Error> {
        let (sql, values) = BookmarkSelect {
            id: params.id,
            tags: params.tags,
            user_id: params.user_id.as_deref(),
            filter: params.filter,
            cursor: params.cursor,
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let bookmark_rows = sqlx::query_as_with::<_, BookmarkRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let (sql, values) = BookmarkTagSelect {
            bookmark_ids: bookmark_rows.iter().map(|e| e.id),
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let tag_rows = sqlx::query_as_with::<_, BookmarkTagRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        let mut tag_row_map = HashMap::<Uuid, Vec<BookmarkTagRow>>::new();

        for row in tag_rows {
            tag_row_map.entry(row.bookmark_id).or_default().push(row);
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

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Bookmark>, Error> {
        let (sql, values) = BookmarkSelectOne { id }
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, BookmarkRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, data: &Bookmark, upsert: Option<BookmarkUpsertType>) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = BookmarkInsert {
            id: data.id,
            link: data.link.as_str(),
            title: &data.title,
            thumbnail_url: data.thumbnail_url.as_ref().map(|e| e.as_str()),
            published_at: data.published_at,
            author: data.author.as_deref(),
            archived_path: data.archived_path.as_deref(),
            user_id: &data.user_id,
            upsert,
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(data.link.clone())
                }
                _ => Error::Database(e),
            })?;

        if let Some(ref tags) = data.tags {
            let (sql, values) = BookmarkTagDelete {
                bookmark_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

            let (sql, values) = BookmarkTagInsert {
                bookmark_id: data.id,
                tags: tags.iter().map(|e| BookmarkTagById {
                    id: e.id,
                    user_id: &e.user_id,
                }),
            }
            .into_insert()
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = BookmarkDelete { id }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub struct BookmarkRow {
    pub id: Uuid,
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

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link.parse().unwrap(),
            title: value.title,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: None,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct BookmarkTagRow {
    pub bookmark_id: Uuid,
    pub id: Uuid,
    pub title: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BookmarkTagRow> for Tag {
    fn from(value: BookmarkTagRow) -> Self {
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

pub struct BookmarkRowWithTagRows {
    pub bookmark: BookmarkRow,
    pub tags: Option<Vec<BookmarkTagRow>>,
}

impl From<BookmarkRowWithTagRows> for Bookmark {
    fn from(value: BookmarkRowWithTagRows) -> Self {
        Self {
            id: value.bookmark.id,
            link: value.bookmark.link.parse().unwrap(),
            title: value.bookmark.title,
            thumbnail_url: value.bookmark.thumbnail_url.and_then(|e| e.parse().ok()),
            published_at: value.bookmark.published_at,
            author: value.bookmark.author,
            archived_path: value.bookmark.archived_path,
            user_id: value.bookmark.user_id,
            created_at: value.bookmark.created_at,
            updated_at: value.bookmark.updated_at,
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}
