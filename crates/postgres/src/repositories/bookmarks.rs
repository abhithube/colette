use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::{self, FindOneParams},
    Bookmark,
};
use colette_database::bookmarks::UpdateParams;
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
    async fn find_many(&self, params: BookmarksFindManyParams) -> Result<Vec<Bookmark>, Error> {
        let bookmarks = queries::bookmarks::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmarks)
    }

    async fn create(&self, data: BookmarksCreateData) -> Result<Bookmark, Error> {
        let bookmark = queries::bookmarks::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<Bookmark, Error> {
        let bookmark = queries::bookmarks::update(
            &self.pool,
            UpdateParams {
                id: &params.id,
                profile_id: &params.profile_id,
                custom_title: data.custom_title.as_deref(),
                custom_thumbnail_url: data.custom_thumbnail_url.as_deref(),
                custom_published_at: data.custom_published_at.as_ref(),
                custom_author: data.custom_author.as_deref(),
            },
        )
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
