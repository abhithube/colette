use std::collections::HashMap;

use chrono::Utc;
use colette_core::{
    Bookmark, Tag,
    bookmark::{BookmarkParams, BookmarkRepository, Error, ImportBookmarksData},
};
use colette_query::{
    Dialect, IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark::{BookmarkBase, BookmarkDelete, BookmarkInsert, BookmarkSelect, BookmarkUpdate},
    bookmark_tag::{BookmarkTagBase, BookmarkTagDelete, BookmarkTagInsert},
    tag::{TagBase, TagInsert, TagSelect},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteBookmarkRepository {
    pool: Pool,
}

impl SqliteBookmarkRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn query(&self, params: BookmarkParams) -> Result<Vec<Bookmark>, Error> {
        let client = self.pool.get().await?;

        let bookmarks = client
            .interact(move |conn| {
                let (sql, values) = BookmarkSelect {
                    id: params.id,
                    filter: params.filter,
                    tags: params.tags,
                    user_id: params.user_id,
                    cursor: params.cursor,
                    limit: params.limit.map(|e| e as u64),
                    with_tags: params.with_tags,
                    dialect: Dialect::Sqlite,
                }
                .into_select()
                .build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Bookmark>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(bookmarks)
    }

    async fn save(&self, data: &Bookmark) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let tx = conn.transaction()?;

                {
                    let (sql, values) = BookmarkInsert {
                        bookmarks: [BookmarkBase {
                            id: data.id,
                            link: data.link.as_str(),
                            title: &data.title,
                            thumbnail_url: data.thumbnail_url.as_ref().map(|e| e.as_str()),
                            published_at: data.published_at,
                            author: data.author.as_deref(),
                            archived_path: data.archived_path.as_deref(),
                            created_at: data.created_at,
                            updated_at: data.updated_at,
                        }],
                        user_id: data.user_id,
                        upsert: false,
                    }
                    .into_insert()
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.execute_prepared(&sql, &values).map_err(|e| {
                        match e.sqlite_error().map(|e| e.extended_code) {
                            Some(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) => {
                                Error::Conflict(data.link.clone())
                            }
                            _ => Error::SqliteClient(e),
                        }
                    })?;
                }

                if let Some(ref tags) = data.tags {
                    let (sql, values) = BookmarkTagDelete {
                        bookmark_id: data.id,
                        tag_ids: tags.iter().map(|e| e.id),
                    }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.execute_prepared(&sql, &values)?;

                    if !tags.is_empty() {
                        let (sql, values) = BookmarkTagInsert {
                            bookmark_tags: [BookmarkTagBase {
                                bookmark_id: data.id,
                                tag_ids: tags.iter().map(|e| e.id),
                            }],
                            user_id: data.user_id,
                        }
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);

                        tx.execute_prepared(&sql, &values)?;
                    }
                }

                tx.commit()?;

                Ok::<_, Error>(())
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn upsert(&self, data: &Bookmark) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let (sql, values) = BookmarkInsert {
                    bookmarks: [BookmarkBase {
                        id: data.id,
                        link: data.link.as_str(),
                        title: &data.title,
                        thumbnail_url: data.thumbnail_url.as_ref().map(|e| e.as_str()),
                        published_at: data.published_at,
                        author: data.author.as_deref(),
                        archived_path: data.archived_path.as_deref(),
                        created_at: data.created_at,
                        updated_at: data.updated_at,
                    }],
                    user_id: data.user_id,
                    upsert: true,
                }
                .into_insert()
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn set_archived_path(
        &self,
        bookmark_id: Uuid,
        archived_path: Option<String>,
    ) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = BookmarkUpdate {
                    id: bookmark_id,
                    archived_path: Some(archived_path.as_deref()),
                    updated_at: Utc::now(),
                }
                .into_update()
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = BookmarkDelete {
                    id: Some(id),
                    ..Default::default()
                }
                .into_delete()
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn import(&self, data: ImportBookmarksData) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let tx = conn.transaction()?;

                let mut tag_map = {
                    let (sql, values) = TagSelect {
                        user_id: Some(data.user_id),
                        ..Default::default()
                    }
                    .into_select()
                    .build_rusqlite(SqliteQueryBuilder);
                    let tags = tx.query_prepared::<Tag>(&sql, &values)?;

                    tags.into_iter()
                        .map(|e| (e.title, e.id))
                        .collect::<HashMap<_, _>>()
                };

                {
                    let mut tags = Vec::<TagBase>::new();

                    for tag in data.tags.iter() {
                        if !tag_map.contains_key(&tag.title) {
                            let id = Uuid::new_v4();

                            tags.push(TagBase {
                                id,
                                title: &tag.title,
                                created_at: tag.created_at,
                                updated_at: tag.updated_at,
                            });

                            tag_map.insert(tag.title.clone(), id);
                        }
                    }

                    if !tags.is_empty() {
                        let (sql, values) = TagInsert {
                            tags,
                            user_id: data.user_id,
                            upsert: true,
                        }
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);
                        tx.execute_prepared(&sql, &values)?;
                    }
                }

                let mut bookmark_map = {
                    let (sql, values) = BookmarkSelect {
                        user_id: Some(data.user_id),
                        dialect: Dialect::Sqlite,
                        ..Default::default()
                    }
                    .into_select()
                    .build_rusqlite(SqliteQueryBuilder);
                    let tags = tx.query_prepared::<Bookmark>(&sql, &values)?;

                    tags.into_iter()
                        .map(|e| (e.link, e.id))
                        .collect::<HashMap<_, _>>()
                };

                {
                    let mut bookmarks = Vec::<BookmarkBase>::new();
                    let mut bookmark_tags = Vec::<BookmarkTagBase<Vec<Uuid>>>::new();

                    for bookmark in data.bookmarks.iter() {
                        if !bookmark_map.contains_key(&bookmark.link) {
                            let id = Uuid::new_v4();

                            bookmarks.push(BookmarkBase {
                                id: bookmark.id,
                                link: bookmark.link.as_str(),
                                title: &bookmark.title,
                                thumbnail_url: bookmark.thumbnail_url.as_ref().map(|e| e.as_str()),
                                published_at: bookmark.published_at,
                                author: bookmark.author.as_deref(),
                                archived_path: bookmark.archived_path.as_deref(),
                                created_at: bookmark.created_at,
                                updated_at: bookmark.updated_at,
                            });

                            bookmark_map.insert(bookmark.link.clone(), id);
                        }

                        if let Some(ref tags) = bookmark.tags
                            && let Some(bookmark_id) = bookmark_map.get(&bookmark.link).copied()
                        {
                            let tag_ids = tags
                                .iter()
                                .flat_map(|e| tag_map.get(&e.title).copied())
                                .collect();

                            bookmark_tags.push(BookmarkTagBase {
                                bookmark_id,
                                tag_ids,
                            });
                        }
                    }

                    if !bookmarks.is_empty() {
                        let (sql, values) = BookmarkInsert {
                            bookmarks,
                            user_id: data.user_id,
                            upsert: true,
                        }
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);
                        tx.execute_prepared(&sql, &values)?;
                    }

                    if !bookmark_tags.is_empty() {
                        let (sql, values) = BookmarkTagInsert {
                            bookmark_tags,
                            user_id: data.user_id,
                        }
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);
                        tx.execute_prepared(&sql, &values)?;
                    }
                };

                tx.commit()?;

                Ok::<_, Error>(())
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Bookmark {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            link: value.get_unwrap::<_, String>("link").parse().unwrap(),
            title: value.get_unwrap("title"),
            thumbnail_url: value
                .get_unwrap::<_, Option<String>>("thumbnail_url")
                .and_then(|e| e.parse().ok()),
            published_at: value.get_unwrap("published_at"),
            archived_path: value.get_unwrap("archived_path"),
            author: value.get_unwrap("author"),
            user_id: value.get_unwrap("user_id"),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
            tags: value
                .get::<_, String>("tags")
                .map(|e| serde_json::from_str(&e).unwrap())
                .ok(),
        }
    }
}
