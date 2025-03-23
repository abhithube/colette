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
use libsql::{Connection, ffi::SQLITE_CONSTRAINT_UNIQUE};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlBookmarkRepository {
    conn: Connection,
}

impl LibsqlBookmarkRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for LibsqlBookmarkRepository {
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
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut bookmark_rows = Vec::<BookmarkRow>::new();
        while let Some(row) = rows.next().await? {
            bookmark_rows.push(libsql::de::from_row(&row)?);
        }

        let (sql, values) = BookmarkTagSelect {
            bookmark_ids: bookmark_rows.iter().map(|e| e.id),
        }
        .into_select()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut tag_rows = Vec::<BookmarkTagRow>::new();
        while let Some(row) = rows.next().await? {
            tag_rows.push(libsql::de::from_row(&row)?);
        }

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
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row::<BookmarkRow>(&row)?.into()))
    }

    async fn save(&self, data: &Bookmark, upsert: Option<BookmarkUpsertType>) -> Result<(), Error> {
        let tx = self.conn.transaction().await?;

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
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = tx.prepare(&sql).await?;
        stmt.execute(values.into_params())
            .await
            .map_err(|e| match e {
                libsql::Error::SqliteFailure(SQLITE_CONSTRAINT_UNIQUE, _) => {
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
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = tx.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;

            let (sql, values) = BookmarkTagInsert {
                bookmark_id: data.id,
                tags: tags.iter().map(|e| BookmarkTagById {
                    id: e.id,
                    user_id: &e.user_id,
                }),
            }
            .into_insert()
            .build_libsql(SqliteQueryBuilder);

            let mut stmt = tx.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = BookmarkDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
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
