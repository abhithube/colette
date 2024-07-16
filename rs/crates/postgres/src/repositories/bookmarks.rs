use colette_core::{
    bookmarks::{BookmarkFindManyParams, BookmarkUpdateData, BookmarksRepository, Error},
    common::{self, FindOneParams},
    Bookmark,
};
use sqlx::PgPool;

use crate::queries;

pub struct BookmarksPostgresRepository {
    pool: PgPool,
}

impl BookmarksPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BookmarksRepository for BookmarksPostgresRepository {
    async fn find_many(&self, params: BookmarkFindManyParams) -> Result<Vec<Bookmark>, Error> {
        let bookmarks = queries::bookmarks::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmarks)
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarkUpdateData,
    ) -> Result<Bookmark, Error> {
        let bookmark = queries::bookmarks::update(&self.pool, (&params).into(), (&data).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(bookmark)
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        queries::bookmarks::delete(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}
