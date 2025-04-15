use chrono::Utc;
use colette_core::{
    Bookmark, Tag,
    bookmark::{BookmarkParams, BookmarkRepository, Error, ImportBookmarksData},
    tag::TagParams,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    bookmark::{BookmarkDelete, BookmarkInsert, BookmarkUpdate},
    bookmark_tag::{BookmarkTagDelete, BookmarkTagInsert},
    tag::TagInsert,
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{Row, error::SqlState, types::Json};
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

        Ok(rows.iter().map(|e| BookmarkRow(e).into()).collect())
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
            let (sql, values) = BookmarkTagDelete {
                bookmark_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params()).await?;

            if !tags.is_empty() {
                let (sql, values) = BookmarkTagInsert {
                    bookmark_id: data.id,
                    user_id: &data.user_id,
                    tag_ids: tags.iter().map(|e| e.id),
                }
                .into_insert()
                .build_postgres(PostgresQueryBuilder);

                let stmt = tx.prepare_cached(&sql).await?;
                tx.execute(&stmt, &values.as_params()).await?;
            }
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
            tags: value.try_get::<_, Json<Vec<Tag>>>("tags").map(|e| e.0).ok(),
        }
    }
}
