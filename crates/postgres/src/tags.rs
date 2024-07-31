use colette_core::{
    common::{FindManyParams, FindOneParams},
    tags::{Error, TagsCreateData, TagsRepository, TagsUpdateData},
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl TagsRepository for PostgresRepository {
    async fn find_many_tags(
        &self,
        params: FindManyParams,
    ) -> Result<Vec<colette_core::Tag>, Error> {
        sqlx::query_file_as!(Tag, "queries/tags/find_many.sql", params.profile_id)
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(colette_core::Tag::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_tag(&self, params: FindOneParams) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(
            Tag,
            "queries/tags/find_one.sql",
            params.id,
            params.profile_id
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Tag::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn create_tag(&self, data: TagsCreateData) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(Tag, "queries/tags/insert.sql", data.title, data.profile_id)
            .fetch_one(&self.pool)
            .await
            .map(colette_core::Tag::from)
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn update_tag(
        &self,
        params: FindOneParams,
        data: TagsUpdateData,
    ) -> Result<colette_core::Tag, Error> {
        sqlx::query_file_as!(
            Tag,
            "queries/tags/update.sql",
            params.id,
            params.profile_id,
            data.title
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Tag::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn delete_tag(&self, params: FindOneParams) -> Result<(), Error> {
        let result = sqlx::query_file!("queries/tags/delete.sql", params.id, params.profile_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, sqlx::Type)]
pub(crate) struct Tag {
    id: Uuid,
    title: String,
}

impl From<Tag> for colette_core::Tag {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            title: value.title,
        }
    }
}