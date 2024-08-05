use colette_core::{
    profiles::{
        Error, ProfilesCreateData, ProfilesFindByIdParams, ProfilesFindManyParams,
        ProfilesFindOneParams, ProfilesRepository, ProfilesUpdateData, StreamProfile,
    },
    Profile,
};
use futures::{stream::BoxStream, StreamExt};

use crate::PostgresRepository;

#[async_trait::async_trait]
impl ProfilesRepository for PostgresRepository {
    async fn find_many_profiles(
        &self,
        params: ProfilesFindManyParams,
    ) -> Result<Vec<Profile>, Error> {
        sqlx::query_file_as!(Profile, "queries/profiles/find_many.sql", params.user_id,)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_profile(&self, params: ProfilesFindOneParams) -> Result<Profile, Error> {
        match params {
            ProfilesFindOneParams::ById(params) => sqlx::query_file_as!(
                Profile,
                "queries/profiles/find_one.sql",
                params.id,
                params.user_id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            }),
            ProfilesFindOneParams::Default { user_id } => {
                sqlx::query_file_as!(Profile, "queries/profiles/find_default.sql", user_id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))
            }
        }
    }

    async fn create_profile(&self, data: ProfilesCreateData) -> Result<Profile, Error> {
        sqlx::query_file_as!(
            Profile,
            "queries/profiles/insert.sql",
            data.title,
            data.image_url,
            false,
            data.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn update_profile(
        &self,
        params: ProfilesFindByIdParams,
        data: ProfilesUpdateData,
    ) -> Result<Profile, Error> {
        sqlx::query_file_as!(
            Profile,
            "queries/profiles/update.sql",
            params.id,
            params.user_id,
            data.title,
            data.image_url
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn delete_profile(&self, params: ProfilesFindByIdParams) -> Result<(), Error> {
        let is_default =
            sqlx::query_file_scalar!("queries/profiles/delete.sql", params.id, params.user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(params.id),
                    _ => Error::Unknown(e.into()),
                })?;

        if is_default {
            return Err(Error::DeletingDefault);
        }

        Ok(())
    }

    fn stream_profiles(&self, feed_id: i32) -> BoxStream<Result<StreamProfile, Error>> {
        Box::pin(
            sqlx::query_file_as!(StreamProfile, "queries/profiles/stream.sql", feed_id)
                .fetch(&self.pool)
                .map(|e| e.map_err(|e| Error::Unknown(e.into()))),
        )
    }
}
