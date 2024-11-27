use std::sync::Arc;

use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagUpdateData},
    Tag,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::{D1Binder, D1Error};

#[derive(Clone)]
pub struct D1TagRepository {
    db: Arc<D1Database>,
}

impl D1TagRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1TagRepository {
    type Params = TagFindParams;
    type Output = Result<Vec<Tag>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::tag::select(
            params.id,
            params.profile_id,
            params.limit,
            params.cursor,
            params.tag_type,
        )
        .build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<Tag>()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1TagRepository {
    type Data = TagCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();

        let (sql, values) = crate::tag::insert(Some(id), data.title.clone(), data.profile_id)
            .build_d1(SqliteQueryBuilder);

        super::run(&self.db, sql, values)
            .await
            .map_err(|e| match e.into() {
                D1Error::UniqueConstraint => Error::Conflict(data.title),
                e => Error::Unknown(e.into()),
            })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for D1TagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            let (sql, values) = crate::tag::update(params.id, params.profile_id, data.title)
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
impl Deletable for D1TagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::tag::delete_by_id(params.id, params.profile_id)
            .build_d1(SqliteQueryBuilder);

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

impl TagRepository for D1TagRepository {}
