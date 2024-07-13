use async_trait::async_trait;
use colette_core::{
    bookmarks::{BookmarkFindManyParams, BookmarksRepository, Error},
    common, Bookmark,
};
use sqlx::PgPool;

use crate::queries;

#[derive(Clone)]
pub struct BookmarksPostgresRepository {
    pool: PgPool,
}

impl BookmarksPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookmarksRepository for BookmarksPostgresRepository {
    async fn find_many(&self, params: BookmarkFindManyParams) -> Result<Vec<Bookmark>, Error> {
        let feeds = queries::bookmarks::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(feeds)
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
