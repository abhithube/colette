use async_trait::async_trait;
use colette_core::{
    common::SendableStream,
    profiles::{
        Error, ProfileCreateData, ProfileFindByIdParams, ProfileFindManyParams,
        ProfileFindOneParams, ProfileUpdateData, ProfilesRepository,
    },
    Profile,
};
use colette_database::profiles::{SelectDefaultParams, UpdateParams};
use futures::TryStreamExt;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::queries;

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
    async fn find_many(&self, params: ProfileFindManyParams) -> Result<Vec<Profile>, Error> {
        let results = queries::profiles::select_many(&self.pool, (&params).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(results)
    }

    async fn find_one(&self, params: ProfileFindOneParams) -> Result<Profile, Error> {
        let profile = match params {
            ProfileFindOneParams::ById(params) => {
                queries::profiles::select_by_id(&self.pool, (&params).into())
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(params.id),
                        _ => Error::Unknown(e.into()),
                    })?
            }
            ProfileFindOneParams::Default { user_id } => queries::profiles::select_default(
                &self.pool,
                SelectDefaultParams { user_id: &user_id },
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?,
        };

        Ok(profile)
    }

    async fn create(&self, data: ProfileCreateData) -> Result<Profile, Error> {
        let profile = queries::profiles::insert(&self.pool, (&data).into())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(profile)
    }

    async fn update(
        &self,
        params: ProfileFindByIdParams,
        data: ProfileUpdateData,
    ) -> Result<Profile, Error> {
        let profile = queries::profiles::update(
            &self.pool,
            UpdateParams {
                id: &params.id,
                user_id: &params.user_id,
                title: data.title.as_deref(),
                image_url: data.image_url.as_deref(),
            },
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(profile)
    }

    async fn delete(&self, params: ProfileFindByIdParams) -> Result<(), Error> {
        let profile = self
            .find_one(ProfileFindOneParams::Default {
                user_id: params.user_id,
            })
            .await?;

        if profile.id == params.id {
            return Err(Error::DeletingDefault);
        }

        queries::profiles::delete(&self.pool, (&params).into())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }

    fn iterate(&self, feed_id: i64) -> SendableStream<Result<Uuid, Error>> {
        Box::pin(
            queries::profiles::iterate(&self.pool, feed_id).map_err(|e| Error::Unknown(e.into())),
        )
    }
}
