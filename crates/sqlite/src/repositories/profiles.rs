use async_trait::async_trait;
use colette_core::{
    profiles::{
        Error, ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams,
        ProfileFindOneParams, ProfileUpdateData, ProfilesRepository,
    },
    Profile,
};
use colette_database::profiles::SelectDefaultParams;
use sqlx::SqlitePool;

use crate::queries;

#[derive(Clone)]
pub struct ProfilesSqliteRepository {
    pool: SqlitePool,
}

impl ProfilesSqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfilesRepository for ProfilesSqliteRepository {
    async fn find_many(&self, params: ProfileFindManyParams<'_>) -> Result<Vec<Profile>, Error> {
        let results = queries::profiles::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(results)
    }

    async fn find_one(&self, params: ProfileFindOneParams<'_>) -> Result<Profile, Error> {
        let profile = match params {
            ProfileFindOneParams::ById(params) => {
                let id = params.id.to_owned();

                queries::profiles::select_by_id(&self.pool, (&params).into())
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(id),
                        _ => Error::Unknown(e.into()),
                    })?
            }
            ProfileFindOneParams::Default { user_id } => {
                queries::profiles::select_default(&self.pool, SelectDefaultParams { user_id })
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            }
        };

        Ok(profile)
    }

    async fn create(&self, data: ProfileCreateData<'_>) -> Result<Profile, Error> {
        let profile = queries::profiles::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(profile)
    }

    async fn update(
        &self,
        params: ProfileFindByIdParams<'_>,
        data: ProfileUpdateData<'_>,
    ) -> Result<Profile, Error> {
        let id = params.id.to_owned();

        let profile = queries::profiles::update(&self.pool, (&params).into(), (&data).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(profile)
    }

    async fn delete(&self, params: ProfileFindByIdParams<'_>) -> Result<(), Error> {
        let profile = self
            .find_one(ProfileFindOneParams::Default {
                user_id: params.user_id,
            })
            .await?;

        if profile.id == params.id {
            return Err(Error::DeletingDefault);
        }

        let id = params.id.to_owned();

        queries::profiles::delete(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}
