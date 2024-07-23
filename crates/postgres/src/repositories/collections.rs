use colette_core::{
    collections::{
        CollectionsCreateData, CollectionsFindManyParams, CollectionsRepository,
        CollectionsUpdateData, Error,
    },
    common::{self, FindOneParams},
    Collection,
};
use colette_database::collections::UpdateParams;
use sqlx::PgPool;

use crate::queries;

pub struct CollectionsPostgresRepository {
    pool: PgPool,
}

impl CollectionsPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CollectionsRepository for CollectionsPostgresRepository {
    async fn find_many(&self, params: CollectionsFindManyParams) -> Result<Vec<Collection>, Error> {
        let collections = queries::collections::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(collections)
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Collection, Error> {
        let collection = queries::collections::select_by_id(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(collection)
    }

    async fn create(&self, data: CollectionsCreateData) -> Result<Collection, Error> {
        let collection = queries::collections::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(collection)
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: CollectionsUpdateData,
    ) -> Result<Collection, Error> {
        let collection = queries::collections::update(
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

        Ok(collection)
    }

    async fn delete(&self, params: common::FindOneParams) -> Result<(), Error> {
        queries::collections::delete(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}
