use colette_core::{
    bookmarks::{
        BookmarksCreateData, BookmarksFindManyParams, BookmarksRepository, BookmarksUpdateData,
        Error,
    },
    common::FindOneParams,
};
use sqlx::types::Json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::{convert_chrono_to_time, convert_time_to_chrono},
    tags::Tag,
    PostgresRepository,
};

#[async_trait::async_trait]
impl BookmarksRepository for PostgresRepository {
    async fn find_many_bookmarks(
        &self,
        params: BookmarksFindManyParams,
    ) -> Result<Vec<colette_core::Bookmark>, Error> {
        sqlx::query_file_as!(
            Bookmark,
            "queries/bookmarks/find_many.sql",
            params.profile_id,
            params.limit,
            params.tags.as_deref()
        )
        .fetch_all(&self.pool)
        .await
        .map(|e| e.into_iter().map(colette_core::Bookmark::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_bookmark(
        &self,
        params: FindOneParams,
    ) -> Result<colette_core::Bookmark, Error> {
        sqlx::query_file_as!(
            Bookmark,
            "queries/bookmarks/find_one.sql",
            params.id,
            params.profile_id
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Bookmark::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn create_bookmark(
        &self,
        data: BookmarksCreateData,
    ) -> Result<colette_core::Bookmark, Error> {
        sqlx::query_file_as!(
            Bookmark,
            "queries/bookmarks/insert.sql",
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published.map(convert_chrono_to_time),
            data.bookmark.author,
            data.profile_id
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Bookmark::from)
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn update_bookmark(
        &self,
        params: FindOneParams,
        data: BookmarksUpdateData,
    ) -> Result<colette_core::Bookmark, Error> {
        match data.tags {
            Some(tags) => sqlx::query_file_as!(
                Bookmark,
                "queries/bookmarks/update.sql",
                params.id,
                params.profile_id,
                &tags
            )
            .fetch_one(&self.pool)
            .await
            .map(colette_core::Bookmark::from)
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            }),
            None => self.find_one_bookmark(params).await,
        }
    }

    async fn delete_bookmark(&self, params: FindOneParams) -> Result<(), Error> {
        let result =
            sqlx::query_file!("queries/bookmarks/delete.sql", params.id, params.profile_id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Bookmark {
    id: Uuid,
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<OffsetDateTime>,
    author: Option<String>,
    tags: Json<Vec<Tag>>,
}

impl From<Bookmark> for colette_core::Bookmark {
    fn from(value: Bookmark) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at.map(convert_time_to_chrono),
            author: value.author,
            tags: value
                .tags
                .0
                .into_iter()
                .map(colette_core::Tag::from)
                .collect(),
        }
    }
}
