use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{BookmarkParams, BookmarkRepository, Error, ImportBookmarksData},
    tag::TagParams,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark::{BookmarkDelete, BookmarkInsert, BookmarkUpdate},
    bookmark_tag::{BookmarkTagDelete, BookmarkTagInsert, BookmarkTagSelect},
    tag::TagInsert,
};
use deadpool_postgres::{Pool, Transaction};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{Row, error::SqlState};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresBookmarkRepository {
    pool: Pool,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn query(&self, params: BookmarkParams) -> Result<Vec<Bookmark>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let bookmark_rows = rows
            .iter()
            .map(|e| Bookmark::from(BookmarkRow(e)))
            .collect::<Vec<_>>();

        let (sql, values) = BookmarkTagSelect {
            bookmark_ids: bookmark_rows.iter().map(|e| e.id),
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        let mut tag_row_map = HashMap::<Uuid, Vec<BookmarkTagRow>>::new();

        let tag_rows = rows.iter().map(BookmarkTagRow::from).collect::<Vec<_>>();
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
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

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
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.link.clone()),
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
        let client = self.pool.get().await?;

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
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }

    async fn set_archived_path(
        &self,
        bookmark_id: Uuid,
        archived_path: Option<String>,
    ) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = BookmarkUpdate {
            id: bookmark_id,
            archived_path: Some(archived_path.as_deref()),
            updated_at: Utc::now(),
        }
        .into_update()
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = BookmarkDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }

    async fn import(&self, data: ImportBookmarksData) -> Result<(), Error> {
        let mut client = self.pool.get().await?;

        let tx = client.transaction().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            data.items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag_id = {
                    let (sql, values) = TagParams {
                        title: Some(item.title.clone()),
                        user_id: Some(data.user_id.clone()),
                        ..Default::default()
                    }
                    .into_select()
                    .build_postgres(PostgresQueryBuilder);

                    let stmt = tx.prepare_cached(&sql).await?;
                    let row = tx.query_opt(&stmt, &values.as_params()).await?;

                    match row {
                        Some(row) => row.get("id"),
                        _ => {
                            let (sql, values) = TagInsert {
                                id: Uuid::new_v4(),
                                title: &item.title,
                                user_id: &data.user_id,
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                                upsert: true,
                            }
                            .into_insert()
                            .build_postgres(PostgresQueryBuilder);

                            let stmt = tx.prepare_cached(&sql).await?;
                            let row = tx.query_one(&stmt, &values.as_params()).await?;

                            row.get("id")
                        }
                    }
                };

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let bookmark_id = {
                    let (sql, values) = BookmarkInsert {
                        id: Uuid::new_v4(),
                        link: &link,
                        title: &item.title,
                        thumbnail_url: None,
                        published_at: None,
                        author: None,
                        archived_path: None,
                        user_id: &data.user_id,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        upsert: true,
                    }
                    .into_insert()
                    .build_postgres(PostgresQueryBuilder);

                    let stmt = tx.prepare_cached(&sql).await?;
                    let row = tx.query_one(&stmt, &values.as_params()).await?;

                    row.get("id")
                };

                if let Some(tag_id) = parent_id {
                    let bookmark_tag = BookmarkTagInsert {
                        bookmark_id,
                        user_id: &data.user_id,
                        tag_ids: vec![tag_id],
                    };

                    let (sql, values) = bookmark_tag
                        .into_insert()
                        .build_postgres(PostgresQueryBuilder);

                    let stmt = tx.prepare_cached(&sql).await?;
                    tx.execute(&stmt, &values.as_params()).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}

impl PostgresBookmarkRepository {
    async fn link_tags(
        &self,
        tx: &Transaction<'_>,
        bookmark_id: Uuid,
        user_id: &str,
        tag_ids: impl IntoIterator<Item = Uuid> + Clone,
    ) -> Result<(), Error> {
        let (sql, values) = BookmarkTagDelete {
            bookmark_id,
            tag_ids: tag_ids.clone(),
        }
        .into_delete()
        .build_postgres(PostgresQueryBuilder);

        let stmt = tx.prepare_cached(&sql).await?;
        tx.execute(&stmt, &values.as_params()).await?;

        let (sql, values) = BookmarkTagInsert {
            bookmark_id,
            user_id,
            tag_ids,
        }
        .into_insert()
        .build_postgres(PostgresQueryBuilder);

        let stmt = tx.prepare_cached(&sql).await?;
        tx.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }
}

struct BookmarkRow<'a>(&'a Row);

impl From<BookmarkRow<'_>> for Bookmark {
    fn from(BookmarkRow(value): BookmarkRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            link: value.get::<_, String>("link").parse().unwrap(),
            title: value.get("title"),
            thumbnail_url: value
                .get::<_, Option<String>>("thumbnail_url")
                .and_then(|e| e.parse().ok()),
            published_at: value.get("published_at"),
            archived_path: value.get("archived_path"),
            author: value.get("author"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            tags: None,
        }
    }
}

struct BookmarkTagRow {
    bookmark_id: Uuid,
    id: Uuid,
    title: String,
    user_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<&Row> for BookmarkTagRow {
    fn from(value: &Row) -> Self {
        Self {
            bookmark_id: value.get("bookmark_id"),
            id: value.get("id"),
            title: value.get("title"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}

impl From<BookmarkTagRow> for Tag {
    fn from(value: BookmarkTagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

struct BookmarkRowWithTagRows {
    bookmark: Bookmark,
    tags: Option<Vec<BookmarkTagRow>>,
}

impl From<BookmarkRowWithTagRows> for Bookmark {
    fn from(value: BookmarkRowWithTagRows) -> Self {
        Self {
            id: value.bookmark.id,
            link: value.bookmark.link,
            title: value.bookmark.title,
            thumbnail_url: value.bookmark.thumbnail_url,
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
