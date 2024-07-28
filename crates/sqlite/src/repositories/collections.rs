use async_trait::async_trait;
use colette_core::{
    collections::{CollectionsCreateData, CollectionsRepository, CollectionsUpdateData, Error},
    common::{self, FindManyParams, FindOneParams},
    Collection,
};
use colette_database::collections::UpdateParams;
use sqlx::SqlitePool;

use crate::queries;

pub struct CollectionsSqliteRepository {
    pool: SqlitePool,
}

impl CollectionsSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CollectionsRepository for CollectionsSqliteRepository {
    async fn find_many(&self, params: FindManyParams) -> Result<Vec<Collection>, Error> {
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        queries::collections::update(
            &mut *tx,
            UpdateParams {
                id: params.id,
                profile_id: params.profile_id,
                title: data.title.as_deref(),
            },
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        let collection = queries::collections::select_by_id(&mut *tx, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

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