use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{BookmarkParams, BookmarkRepository, Error},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark::{BookmarkDelete, BookmarkInsert, BookmarkUpdate},
    bookmark_tag::{BookmarkTagDelete, BookmarkTagInsert, BookmarkTagSelect},
};
use libsql::{Connection, Transaction, ffi::SQLITE_CONSTRAINT_UNIQUE};
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
    async fn query(&self, params: BookmarkParams) -> Result<Vec<Bookmark>, Error> {
        let (sql, values) = params.into_select().build_libsql(SqliteQueryBuilder);

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

    async fn save(&self, data: &Bookmark) -> Result<(), Error> {
        let tx = self.conn.transaction().await?;

        {
            let (sql, values) = BookmarkInsert {
                id: data.id,
                link: data.link.as_str(),
                title: &data.title,
                thumbnail_url: data.thumbnail_url.as_ref().map(|e| e.as_str()),
                published_at: data.published_at,
                author: data.author.as_deref(),
                archived_path: data.archived_path.as_deref(),
                user_id: &data.user_id,
                created_at: data.created_at,
                updated_at: data.updated_at,
                upsert: false,
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
        }

        if let Some(ref tags) = data.tags {
            self.link_tags(&tx, data.id, &data.user_id, tags.iter().map(|e| e.id))
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn upsert(&self, data: &Bookmark) -> Result<(), Error> {
        let (sql, values) = BookmarkInsert {
            id: data.id,
            link: data.link.as_str(),
            title: &data.title,
            thumbnail_url: data.thumbnail_url.as_ref().map(|e| e.as_str()),
            published_at: data.published_at,
            author: data.author.as_deref(),
            archived_path: data.archived_path.as_deref(),
            user_id: &data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
            upsert: true,
        }
        .into_insert()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }

    async fn set_archived_path(
        &self,
        bookmark_id: Uuid,
        archived_path: Option<String>,
    ) -> Result<(), Error> {
        let (sql, values) = BookmarkUpdate {
            id: bookmark_id,
            archived_path: Some(archived_path.as_deref()),
            updated_at: Utc::now(),
        }
        .into_update()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

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

impl LibsqlBookmarkRepository {
    async fn link_tags(
        &self,
        tx: &Transaction,
        bookmark_id: Uuid,
        user_id: &str,
        tag_ids: impl IntoIterator<Item = Uuid> + Clone,
    ) -> Result<(), Error> {
        let (sql, values) = BookmarkTagDelete {
            bookmark_id,
            tag_ids: tag_ids.clone(),
        }
        .into_delete()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = tx.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        let (sql, values) = BookmarkTagInsert {
            bookmark_id,
            user_id,
            tag_ids,
        }
        .into_insert()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = tx.prepare(&sql).await?;
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
    pub published_at: Option<i64>,
    pub archived_path: Option<String>,
    pub author: Option<String>,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link.parse().unwrap(),
            title: value.title,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            published_at: value
                .published_at
                .and_then(|e| DateTime::from_timestamp(e, 0)),
            author: value.author,
            archived_path: value.archived_path,
            user_id: value.user_id,
            created_at: DateTime::from_timestamp(value.created_at, 0).unwrap(),
            updated_at: DateTime::from_timestamp(value.updated_at, 0).unwrap(),
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
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<BookmarkTagRow> for Tag {
    fn from(value: BookmarkTagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: DateTime::from_timestamp(value.created_at, 0).unwrap(),
            updated_at: DateTime::from_timestamp(value.updated_at, 0).unwrap(),
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
            published_at: value
                .bookmark
                .published_at
                .and_then(|e| DateTime::from_timestamp(e, 0)),
            author: value.bookmark.author,
            archived_path: value.bookmark.archived_path,
            user_id: value.bookmark.user_id,
            created_at: DateTime::from_timestamp(value.bookmark.created_at, 0).unwrap(),
            updated_at: DateTime::from_timestamp(value.bookmark.updated_at, 0).unwrap(),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}
