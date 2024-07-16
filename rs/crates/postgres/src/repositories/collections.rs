use colette_core::{
    collections::{CollectionCreateData, CollectionFindManyParams, CollectionsRepository, Error},
    common, Collection,
};
use sqlx::PgPool;

use crate::queries;

#[derive(Clone)]
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
    async fn find_many(&self, params: CollectionFindManyParams) -> Result<Vec<Collection>, Error> {
        let feeds = queries::collections::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(feeds)
    }

    async fn find_one(&self, params: common::FindOneParams) -> Result<Collection, Error> {
        let feed = queries::collections::select_by_id(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(feed)
    }

    async fn create(&self, data: CollectionCreateData) -> Result<Collection, Error> {
        let collection = queries::collections::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

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