use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Bookmark, Tag,
    bookmark::{BookmarkParams, BookmarkRepository, Error, ImportBookmarksData},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark::{BookmarkBase, BookmarkDelete, BookmarkInsert, BookmarkSelect, BookmarkUpdate},
    bookmark_tag::{BookmarkTagBase, BookmarkTagDelete, BookmarkTagInsert},
    tag::{TagBase, TagInsert, TagSelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::postgres::{DbUrl, tag::TagRow};

#[derive(Debug, Clone)]
pub struct PostgresBookmarkRepository {
    pool: PgPool,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn query(&self, params: BookmarkParams) -> Result<Vec<Bookmark>, Error> {
        let (sql, values) = BookmarkSelect {
            id: params.id,
            filter: params.filter,
            tags: params.tags,
            user_id: params.user_id,
            cursor: params.cursor,
            limit: params.limit.map(|e| e as u64),
            with_tags: params.with_tags,
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, BookmarkRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| println!("{e}"))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Bookmark) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

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
            .build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.link.clone())
                    }
                    _ => Error::Sqlx(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = BookmarkTagDelete {
                bookmark_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

            if !tags.is_empty() {
                let (sql, values) = BookmarkTagInsert {
                    bookmark_tags: [BookmarkTagBase {
                        bookmark_id: data.id,
                        tag_ids: tags.iter().map(|e| e.id),
                    }],
                    user_id: data.user_id,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn upsert(&self, data: &Bookmark) -> Result<(), Error> {
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
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

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
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = BookmarkDelete {
            id: Some(id),
            ..Default::default()
        }
        .into_delete()
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn import(&self, data: ImportBookmarksData) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

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

            if !tags.is_empty() {
                let (sql, values) = TagInsert {
                    tags,
                    user_id: data.user_id,
                    upsert: true,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }

            let (sql, values) = TagSelect {
                titles: Some(titles),
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_select()
            .build_sqlx(PostgresQueryBuilder);
            let rows = sqlx::query_as_with::<_, TagRow, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await?;

            rows.into_iter()
                .map(|e| (e.title, e.id))
                .collect::<HashMap<_, _>>()
        };

        let mut bookmark_map = {
            let (sql, values) = BookmarkSelect {
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_select()
            .build_sqlx(PostgresQueryBuilder);
            let rows = sqlx::query_as_with::<_, BookmarkRow, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await?;

            rows.into_iter()
                .map(|e| (e.link.0, e.id))
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
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }

            if !bookmark_tags.is_empty() {
                let (sql, values) = BookmarkTagInsert {
                    bookmark_tags,
                    user_id: data.user_id,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }
        };

        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct BookmarkRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    thumbnail_url: Option<DbUrl>,
    published_at: Option<DateTime<Utc>>,
    archived_path: Option<String>,
    author: Option<String>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[sqlx(default)]
    tags: Option<Json<Vec<Tag>>>,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link.0,
            title: value.title,
            thumbnail_url: value.thumbnail_url.map(|e| e.0),
            published_at: value.published_at,
            archived_path: value.archived_path,
            author: value.author,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.0),
        }
    }
}
