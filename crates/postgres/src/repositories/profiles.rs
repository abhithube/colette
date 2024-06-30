use async_trait::async_trait;
use colette_core::{
    profiles::{
        Error, ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams,
        ProfileFindOneParams, ProfileUpdateData, ProfilesRepository,
    },
    Profile,
};
use sqlx::PgPool;

use crate::queries::profiles::{self, SelectDefaultParams};

pub struct ProfilesPostgresRepository {
    pool: PgPool,
}

impl ProfilesPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfilesRepository for ProfilesPostgresRepository {
    async fn find_many(&self, params: ProfileFindManyParams) -> Result<Vec<Profile>, Error> {
        let results = profiles::select_many(&self.pool, params.into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(results)
    }

    async fn find_one(&self, params: ProfileFindOneParams) -> Result<Profile, Error> {
        let profile = match params {
            ProfileFindOneParams::ById(params) => {
                let id = params.id.clone();

                profiles::select_by_id(&self.pool, params.into())
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(id),
                        _ => Error::Unknown(e.into()),
                    })?
            }
            ProfileFindOneParams::Default { user_id } => {
                profiles::select_default(&self.pool, SelectDefaultParams { user_id })
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            }
        };

        Ok(profile)
    }

    async fn create(&self, data: ProfileCreateData) -> Result<Profile, Error> {
        let profile = profiles::insert(&self.pool, data.into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(profile)
    }

    async fn update(
        &self,
        params: ProfileFindByIdParams,
        data: ProfileUpdateData,
    ) -> Result<Profile, Error> {
        let id = params.id.clone();

        let profile = profiles::update(&self.pool, params.into(), data.into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(profile)
    }

    async fn delete(&self, params: ProfileFindByIdParams) -> Result<(), Error> {
        let id = params.id.clone();

        profiles::delete(&self.pool, params.into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}
