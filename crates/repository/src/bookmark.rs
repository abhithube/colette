use colette_core::{
    bookmark::{
        BookmarkCacheData, BookmarkCreateData, BookmarkFindParams, BookmarkRepository,
        BookmarkUpdateData, ConflictError, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use sqlx::{PgConnection, Pool, Postgres};
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
        crate::query::user_bookmark::select(
            &self.pool,
            params.id,
            params.folder_id,
            params.user_id,
            params.cursor,
            params.limit,
            params.tags,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark_id = {
            crate::query::bookmark::select_by_link(&mut *tx, data.url.clone())
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => {
                        Error::Conflict(ConflictError::NotCached(data.url.clone()))
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        let pb_id = crate::query::user_bookmark::insert(
            &mut *tx,
            data.title,
            data.thumbnail_url,
            data.published_at,
            data.author,
            data.folder_id,
            bookmark_id,
            data.user_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => {
                Error::Conflict(ConflictError::AlreadyExists(data.url))
            }
            _ => Error::Unknown(e.into()),
        })?;

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pb_id, &tags, data.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(pb_id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some()
            || data.thumbnail_url.is_some()
            || data.published_at.is_some()
            || data.author.is_some()
            || data.folder_id.is_some()
        {
            crate::query::user_bookmark::update(
                &mut *tx,
                params.id,
                params.user_id,
                data.title,
                data.thumbnail_url,
                data.published_at,
                data.author,
                data.folder_id,
            )
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, &tags, params.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        crate::query::user_bookmark::delete(&self.pool, params.id, params.user_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error> {
        crate::query::bookmark::insert(
            &self.pool,
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

pub(crate) async fn link_tags(
    conn: &mut PgConnection,
    user_bookmark_id: Uuid,
    tags: &[String],
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    crate::query::user_bookmark_tag::delete_many(&mut *conn, tags, user_id).await?;

    crate::query::tag::insert_many(&mut *conn, tags, user_id).await?;

    crate::query::user_bookmark_tag::insert_many(&mut *conn, user_bookmark_id, tags, user_id).await
}
