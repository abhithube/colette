use std::sync::Arc;

use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Cursor, Error, ProfileCreateData, ProfileIdOrDefaultParams, ProfileIdParams,
        ProfileRepository, ProfileUpdateData,
    },
    Profile,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::D1Binder;

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
    type Params = ProfileIdOrDefaultParams;
    type Output = Result<Profile, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let is_default = params.id.map_or_else(|| Some(true), |_| None);

        let mut profiles =
            find(&self.db, params.id, params.user_id, is_default, None, None).await?;
        if profiles.is_empty() {
            if let Some(id) = params.id {
                return Err(Error::NotFound(id));
            } else {
                return Err(Error::Unknown(anyhow!("couldn't find default profile")));
            }
        }

        Ok(profiles.swap_remove(0))
    }
}

#[async_trait::async_trait]
impl Creatable for D1ProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let (sql, values) = colette_sql::profile::insert(
            Some(Uuid::new_v4()),
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
        .build_d1(SqliteQueryBuilder);

        let id = super::first(&self.db, sql, values, Some("id"))
            .await
            .map_err(|e| {
                println!("{:?}", e);

                Error::Unknown(e.into())
            })?
            .unwrap();

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
            let count = {
                let (sql, values) = colette_sql::profile::update(
                    params.id,
                    params.user_id,
                    data.title,
                    data.image_url,
                )
                .build_d1(SqliteQueryBuilder);

                super::run(&self.db, sql, values)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                1
            };
            if count == 0 {
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
        let mut profiles =
            find(&self.db, Some(params.id), params.user_id, None, None, None).await?;
        if profiles.is_empty() {
            return Err(Error::NotFound(params.id));
        }

        let profile = profiles.swap_remove(0);
        if profile.is_default {
            return Err(Error::DeletingDefault);
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

#[async_trait::async_trait]
impl ProfileRepository for D1ProfileRepository {
    async fn list(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<Profile>, Error> {
        find(&self.db, None, user_id, None, limit, cursor).await
    }
}

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
            is_default: value.is_default.eq(&0),
            user_id: value.user_id,
        }
    }
}

async fn find(
    db: &D1Database,
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<Profile>, Error> {
    let (sql, values) = colette_sql::profile::select(id, user_id, is_default, cursor, limit)
        .build_d1(SqliteQueryBuilder);

    let result = super::all(db, sql, values)
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

    result
        .results::<ProfileSelect>()
        .map(|e| e.into_iter().map(Profile::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
}
