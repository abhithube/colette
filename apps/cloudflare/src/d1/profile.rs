use std::sync::Arc;

use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Error, ProfileCreateData, ProfileFindParams, ProfileIdParams, ProfileRepository,
        ProfileUpdateData,
    },
    Profile,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::{D1Binder, D1Error};

#[derive(Clone)]
pub struct D1ProfileRepository {
    db: Arc<D1Database>,
}

impl D1ProfileRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1ProfileRepository {
    type Params = ProfileFindParams;
    type Output = Result<Vec<Profile>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = colette_sql::profile::select(
            params.id,
            params.user_id,
            params.is_default,
            params.cursor,
            params.limit,
        )
        .build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<ProfileSelect>()
            .map(|e| e.into_iter().map(Profile::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for D1ProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = Uuid::new_v4();

        let (sql, values) = colette_sql::profile::insert(
            Some(id),
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
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
impl Updatable for D1ProfileRepository {
    type Params = ProfileIdParams;
    type Data = ProfileUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() || data.image_url.is_some() {
            let (sql, values) =
                colette_sql::profile::update(params.id, params.user_id, data.title, data.image_url)
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
impl Deletable for D1ProfileRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        {
            let mut profiles = self
                .find(ProfileFindParams {
                    id: Some(params.id),
                    user_id: params.user_id,
                    ..Default::default()
                })
                .await?;
            if profiles.is_empty() {
                return Err(Error::NotFound(params.id));
            }

            let profile = profiles.swap_remove(0);
            if profile.is_default {
                return Err(Error::DeletingDefault);
            }
        }

        {
            let (sql, values) = colette_sql::smart_feed::delete(params.id, params.user_id)
                .build_d1(SqliteQueryBuilder);

            super::run(&self.db, sql, values)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        };

        Ok(())
    }
}

impl ProfileRepository for D1ProfileRepository {}

#[derive(Debug, serde::Deserialize)]
pub struct ProfileSelect {
    pub id: Uuid,
    pub title: String,
    pub image_url: Option<String>,
    pub is_default: u32,
    pub user_id: Uuid,
}

impl From<ProfileSelect> for Profile {
    fn from(value: ProfileSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            is_default: value.is_default.eq(&1),
            user_id: value.user_id,
        }
    }
}
