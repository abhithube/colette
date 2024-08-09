use colette_core::{
    profiles::{
        Error, ProfilesCreateData, ProfilesFindByIdParams, ProfilesFindManyParams,
        ProfilesFindOneParams, ProfilesRepository, ProfilesUpdateData, StreamProfile,
    },
    Profile,
};
use colette_entities::profile;
use futures::{stream::BoxStream, StreamExt};
use sea_orm::{
    ColumnTrait, EntityTrait, ModelTrait, QueryFilter, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl ProfilesRepository for PostgresRepository {
    async fn find_many_profiles(
        &self,
        params: ProfilesFindManyParams,
    ) -> Result<Vec<Profile>, Error> {
        sqlx::query_file_as!(Profile, "queries/profiles/find_many.sql", params.user_id,)
            .fetch_all(self.db.get_postgres_connection_pool())
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
            .fetch_one(self.db.get_postgres_connection_pool())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            }),
            ProfilesFindOneParams::Default { user_id } => {
                sqlx::query_file_as!(Profile, "queries/profiles/find_default.sql", user_id)
                    .fetch_one(self.db.get_postgres_connection_pool())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))
            }
        }
    }

    async fn create_profile(&self, data: ProfilesCreateData) -> Result<Profile, Error> {
        sqlx::query_file_as!(
            Profile,
            "queries/profiles/insert.sql",
            Uuid::new_v4(),
            data.title,
            data.image_url,
            false,
            data.user_id
        )
        .fetch_one(self.db.get_postgres_connection_pool())
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
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn delete_profile(&self, params: ProfilesFindByIdParams) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(profile) = profile::Entity::find_by_id(params.id)
                        .filter(profile::Column::UserId.eq(params.user_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };

                    if profile.is_default {
                        return Err(Error::DeletingDefault);
                    }

                    profile
                        .delete(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }

    fn stream_profiles(&self, feed_id: i32) -> BoxStream<Result<StreamProfile, Error>> {
        Box::pin(
            sqlx::query_file_as!(StreamProfile, "queries/profiles/stream.sql", feed_id)
                .fetch(self.db.get_postgres_connection_pool())
                .map(|e| e.map_err(|e| Error::Unknown(e.into()))),
        )
    }
}
