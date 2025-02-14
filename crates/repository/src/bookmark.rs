use colette_core::{
    Bookmark,
    bookmark::{
        BookmarkCreateData, BookmarkFindParams, BookmarkRepository, BookmarkUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresBookmarkRepository {
    pool: Pool<Postgres>,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresBookmarkRepository {
    type Params = BookmarkFindParams;
    type Output = Result<Vec<Bookmark>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let bookmarks = crate::common::select_bookmarks(
            &self.pool,
            params.id,
            params.folder_id,
            params.user_id,
            params.cursor,
            params.limit,
            params.tags,
        )
        .await?;

        Ok(bookmarks)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self.pool.begin().await?;

        let ub_id = sqlx::query_file_scalar!(
            "queries/bookmarks/insert.sql",
            data.url,
            data.title,
            data.thumbnail_url,
            data.published_at,
            data.author,
            data.folder_id,
            data.user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.url),
            _ => Error::Database(e),
        })?;

        if let Some(tags) = data.tags {
            if !tags.is_empty() {
                sqlx::query_file_scalar!(
                    "queries/bookmark_tags/link.sql",
                    ub_id,
                    data.user_id,
                    &tags
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(ub_id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self.pool.begin().await?;

        if data.title.is_some()
            || data.thumbnail_url.is_some()
            || data.published_at.is_some()
            || data.author.is_some()
            || data.archived_url.is_some()
            || data.folder_id.is_some()
        {
            let (has_title, title) = match data.title {
                Some(title) => (true, title),
                None => (false, None),
            };
            let (has_thumbnail_url, thumbnail_url) = match data.thumbnail_url {
                Some(thumbnail_url) => (true, thumbnail_url),
                None => (false, None),
            };
            let (has_published_at, published_at) = match data.published_at {
                Some(published_at) => (true, published_at),
                None => (false, None),
            };
            let (has_author, author) = match data.author {
                Some(author) => (true, author),
                None => (false, None),
            };
            let (has_archived_url, archived_url) = match data.archived_url {
                Some(archived_url) => (true, archived_url),
                None => (false, None),
            };
            let (has_folder, folder_id) = match data.folder_id {
                Some(folder_id) => (true, folder_id),
                None => (false, None),
            };

            sqlx::query_file!(
                "queries/bookmarks/update.sql",
                params.id,
                params.user_id,
                has_title,
                title,
                has_thumbnail_url,
                thumbnail_url,
                has_published_at,
                published_at,
                has_author,
                author,
                has_archived_url,
                archived_url,
                has_folder,
                folder_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;
        }

        if let Some(tags) = data.tags {
            if !tags.is_empty() {
                sqlx::query_file_scalar!(
                    "queries/bookmark_tags/link.sql",
                    params.id,
                    params.user_id,
                    &tags
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        sqlx::query_file!("queries/bookmarks/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;

        Ok(())
    }
}

impl BookmarkRepository for PostgresBookmarkRepository {}
