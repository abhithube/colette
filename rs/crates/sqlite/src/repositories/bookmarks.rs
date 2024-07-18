use async_trait::async_trait;
use colette_core::{
    bookmarks::{
        BookmarkCreateData, BookmarkFindManyParams, BookmarkUpdateData, BookmarksRepository, Error,
    },
    common::{self, FindOneParams},
    Bookmark,
};
use colette_database::{
    bookmarks::UpdateParams, collections::SelectDefaultParams, SelectByIdParams,
};
use sqlx::SqlitePool;
use uuid::Uuid;

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
        let bookmarks = queries::bookmarks::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmarks)
    }

    async fn create(&self, data: BookmarkCreateData) -> Result<Bookmark, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let collection = match data.collection_id {
            Some(id) => queries::collections::select_by_id(
                &mut *tx,
                SelectByIdParams {
                    id: &id,
                    profile_id: &data.profile_id,
                },
            )
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Unknown(e.into()),
            })?,
            None => queries::collections::select_default(
                &mut *tx,
                SelectDefaultParams {
                    profile_id: &data.profile_id,
                },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?,
        };

        let id = Uuid::new_v4();
        queries::bookmarks::insert(
            &mut *tx,
            queries::bookmarks::InsertParams {
                id,
                link: &data.link,
                title: &data.title,
                thumbnail_url: data.thumbnail_url.as_deref(),
                published_at: data.published_at.as_ref(),
                author: data.author.as_deref(),
                collection_id: &collection.id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark = queries::bookmarks::select_by_id(
            &mut *tx,
            SelectByIdParams {
                id: &id,
                profile_id: &data.profile_id,
            },
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: BookmarkUpdateData,
    ) -> Result<Bookmark, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        queries::bookmarks::update(
            &mut *tx,
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

        let bookmark = queries::bookmarks::select_by_id(&mut *tx, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

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
