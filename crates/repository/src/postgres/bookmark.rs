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
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use tokio_postgres::{error::SqlState, types::Json};
use uuid::Uuid;

use super::{PgRow, PreparedClient as _};

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

        let (sql, values) = BookmarkSelect {
            id: params.id,
            filter: params.filter,
            tags: params.tags,
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit,
            with_tags: params.with_tags,
            dialect: Dialect::Postgres,
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let bookmarks = client.query_prepared::<Bookmark>(&sql, &values).await?;

        Ok(bookmarks)
    }

    async fn save(&self, data: &Bookmark) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

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
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values)
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.link.clone()),
                    _ => Error::PostgresClient(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = BookmarkTagDelete {
                bookmark_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            if !tags.is_empty() {
                let (sql, values) = BookmarkTagInsert {
                    bookmark_tags: [BookmarkTagBase {
                        bookmark_id: data.id,
                        tag_ids: tags.iter().map(|e| e.id),
                    }],
                    user_id: data.user_id,
                }
                .into_insert()
                .build_postgres(PostgresQueryBuilder);
                tx.execute_prepared(&sql, &values).await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn upsert(&self, data: &Bookmark) -> Result<(), Error> {
        let client = self.pool.get().await?;

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
        .build_postgres(PostgresQueryBuilder);
        client.execute_prepared(&sql, &values).await?;

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
        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = BookmarkDelete {
            id: Some(id),
            ..Default::default()
        }
        .into_delete()
        .build_postgres(PostgresQueryBuilder);
        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }

    async fn import(&self, data: ImportBookmarksData) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        let tag_map = {
            let mut titles = Vec::<&str>::new();
            let mut tags = Vec::<TagBase>::new();

            for tag in data.tags.iter() {
                titles.push(&tag.title);

                tags.push(TagBase {
                    id: tag.id,
                    title: &tag.title,
                    created_at: tag.created_at,
                    updated_at: tag.updated_at,
                });
            }

            let (sql, values) = TagInsert {
                tags,
                user_id: data.user_id,
                upsert: true,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let (sql, values) = TagSelect {
                titles: Some(titles),
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_select()
            .build_postgres(PostgresQueryBuilder);
            let tags = tx.query_prepared::<Tag>(&sql, &values).await?;

            tags.into_iter()
                .map(|e| (e.title, e.id))
                .collect::<HashMap<_, _>>()
        };

        let mut bookmark_map = {
            let (sql, values) = BookmarkSelect {
                user_id: Some(data.user_id),
                dialect: Dialect::Postgres,
                ..Default::default()
            }
            .into_select()
            .build_postgres(PostgresQueryBuilder);
            let bookmarks = tx.query_prepared::<Bookmark>(&sql, &values).await?;

            bookmarks
                .into_iter()
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

            let (sql, values) = BookmarkInsert {
                bookmarks,
                user_id: data.user_id,
                upsert: true,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let (sql, values) = BookmarkTagInsert {
                bookmark_tags,
                user_id: data.user_id,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;
        };

        tx.commit().await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Bookmark {
    fn from(PgRow(value): PgRow<'_>) -> Self {
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
            tags: value.try_get::<_, Json<Vec<Tag>>>("tags").map(|e| e.0).ok(),
        }
    }
}
