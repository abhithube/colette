use async_trait::async_trait;
use colette_core::{
    collections::{
        CollectionCreateData, CollectionFindManyParams, CollectionUpdateData,
        CollectionsRepository, Error,
    },
    common::{self, FindOneParams},
    Collection,
};
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
    async fn find_many(&self, params: CollectionFindManyParams) -> Result<Vec<Collection>, Error> {
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

    async fn create(&self, data: CollectionCreateData) -> Result<Collection, Error> {
        let collection = queries::collections::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(collection)
    }

    async fn update(
        &self,
        params: FindOneParams,
        data: CollectionUpdateData,
    ) -> Result<Collection, Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        queries::collections::update(&mut *tx, (&params).into(), (&data).into())
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
