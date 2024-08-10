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
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, QueryOrder, Set, SqlErr, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl ProfilesRepository for PostgresRepository {
    async fn find_many_profiles(
        &self,
        params: ProfilesFindManyParams,
    ) -> Result<Vec<Profile>, Error> {
        profile::Entity::find()
            .filter(profile::Column::UserId.eq(params.user_id))
            .order_by_asc(profile::Column::Title)
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Profile::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_profile(&self, params: ProfilesFindOneParams) -> Result<Profile, Error> {
        match params {
            ProfilesFindOneParams::ById(params) => {
                find_by_id(&self.db, params.id, params.user_id).await
            }
            ProfilesFindOneParams::Default { user_id } => {
                let Some(profile) = profile::Entity::find()
                    .filter(profile::Column::UserId.eq(user_id))
                    .filter(profile::Column::IsDefault.eq(true))
                    .one(&self.db)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::Unknown(anyhow::anyhow!(
                        "couldn't find default profile"
                    )));
                };

                Ok(profile.into())
            }
        }
    }

    async fn create_profile(&self, data: ProfilesCreateData) -> Result<Profile, Error> {
        let model = profile::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.clone()),
            image_url: Set(data.image_url),
            user_id: Set(data.user_id),
            ..Default::default()
        };

        let profile = profile::Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(profile.into())
    }

    async fn update_profile(
        &self,
        params: ProfilesFindByIdParams,
        data: ProfilesUpdateData,
    ) -> Result<Profile, Error> {
        self.db
            .transaction::<_, colette_core::Profile, Error>(|txn| {
                Box::pin(async move {
                    let Some(mut model) = profile::Entity::find_by_id(params.id)
                        .filter(profile::Column::UserId.eq(params.user_id))
                        .one(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };
                    let mut active_model = model.clone().into_active_model();

                    if let Some(title) = data.title {
                        active_model.title.set_if_not_equals(title);
                    }
                    if data.image_url.is_some() {
                        active_model.image_url.set_if_not_equals(data.image_url);
                    }

                    if active_model.is_changed() {
                        model = active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(model.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
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

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    user_id: Uuid,
) -> Result<colette_core::Profile, Error> {
    let Some(profile) = profile::Entity::find_by_id(id)
        .filter(profile::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(|e| Error::Unknown(e.into()))?
    else {
        return Err(Error::NotFound(id));
    };

    Ok(profile.into())
}
