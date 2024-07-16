use async_trait::async_trait;
use colette_core::{
    bookmarks::{BookmarkFindManyParams, BookmarkUpdateData, BookmarksRepository, Error},
    common::{self, FindOneParams},
    Bookmark,
};
use sqlx::SqlitePool;

use crate::queries;

pub struct BookmarksSqliteRepository {
    pool: SqlitePool,
}

impl BookmarksSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookmarksRepository for BookmarksSqliteRepository {
    async fn find_many(&self, params: BookmarkFindManyParams) -> Result<Vec<Bookmark>, Error> {
        let feeds = queries::bookmarks::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(feeds)
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarkUpdateData,
    ) -> Result<Bookmark, Error> {
        queries::bookmarks::update(&self.pool, (&params).into(), (&data).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        let bookmark = queries::bookmarks::select_by_id(&self.pool, (&params).into())
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
