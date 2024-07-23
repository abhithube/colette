use async_trait::async_trait;
use colette_core::{
    common::{self, FindManyParams, FindOneParams},
    tags::{Error, TagsCreateData, TagsRepository, TagsUpdateData},
    Tag,
};
use colette_database::tags::UpdateParams;
use sqlx::SqlitePool;

use crate::queries;

pub struct TagsSqliteRepository {
    pool: SqlitePool,
}

impl TagsSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagsRepository for TagsSqliteRepository {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Tag>, Error> {
        let tags = queries::tags::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(tags)
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Tag, Error> {
        let tag = queries::tags::select_by_id(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(tag)
    }

    async fn create(&self, data: TagsCreateData) -> Result<Tag, Error> {
        let tag = queries::tags::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }

    async fn update(&self, params: FindOneParams, data: TagsUpdateData) -> Result<Tag, Error> {
        let tag = queries::tags::update(
            &self.pool,
            UpdateParams {
                id: &params.id,
                profile_id: &params.profile_id,
                title: data.title.as_deref(),
            },
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(tag)
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        queries::tags::delete(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}
