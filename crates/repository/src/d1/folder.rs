use std::sync::Arc;

use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    folder::{Error, FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
    Folder,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::D1Binder;

#[derive(Clone)]
pub struct D1FolderRepository {
    db: Arc<D1Database>,
}

impl D1FolderRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1FolderRepository {
    type Params = FolderFindParams;
    type Output = Result<Vec<Folder>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::folder::select(
            params.id,
            params.user_id,
            params.parent_id,
            params.limit,
            params.cursor,
        )
        .build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<Folder>()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1FolderRepository {
    type Data = FolderCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();

        let (sql, values) =
            crate::folder::insert(Some(id), data.title.clone(), data.parent_id, data.user_id)
                .build_d1(SqliteQueryBuilder);

        super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for D1FolderRepository {
    type Params = IdParams;
    type Data = FolderUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            let (sql, values) =
                crate::folder::update(params.id, params.user_id, data.title, data.parent_id)
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
impl Deletable for D1FolderRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::folder::delete_by_id(params.id, params.user_id).build_d1(SqliteQueryBuilder);

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

impl FolderRepository for D1FolderRepository {}
