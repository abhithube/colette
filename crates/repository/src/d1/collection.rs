use std::sync::Arc;

use colette_core::{
    collection::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
        Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Collection,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::D1Binder;

#[derive(Clone)]
pub struct D1CollectionRepository {
    db: Arc<D1Database>,
}

impl D1CollectionRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1CollectionRepository {
    type Params = CollectionFindParams;
    type Output = Result<Vec<Collection>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::collection::select(params.id, params.user_id, params.limit, params.cursor)
                .build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<Collection>()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1CollectionRepository {
    type Data = CollectionCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();

        let (sql, values) = crate::collection::insert(Some(id), data.title.clone(), data.user_id)
            .build_d1(SqliteQueryBuilder);

        super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for D1CollectionRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            let (sql, values) = crate::collection::update(params.id, params.user_id, data.title)
                .build_d1(SqliteQueryBuilder);

            let result = super::run(&self.db, sql, values)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
            let meta = result.meta().unwrap().unwrap();

            if meta.changes.is_none_or(|e| e == 0) {
                return Err(Error::NotFound(params.id));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for D1CollectionRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::collection::delete_by_id(params.id, params.user_id).build_d1(SqliteQueryBuilder);

        let result = super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        let meta = result.meta().unwrap().unwrap();

        if meta.changes.is_none_or(|e| e == 0) {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

impl CollectionRepository for D1CollectionRepository {}
