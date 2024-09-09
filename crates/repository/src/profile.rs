use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Cursor, Error, ProfileCreateData, ProfileIdOrDefaultParams, ProfileIdParams,
        ProfileRepository, ProfileUpdateData,
    },
    Profile,
};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, ModelTrait, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::query;

pub struct ProfileSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl ProfileSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for ProfileSqlRepository {
    type Params = ProfileIdOrDefaultParams;
    type Output = Result<Profile, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params.id {
            Some(id) => find_by_id(&self.db, id, params.user_id).await,
            None => {
                let Some(profile) = query::profile::select_default(&self.db, params.user_id)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::Unknown(anyhow!("couldn't find default profile")));
                };

                Ok(profile.into())
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for ProfileSqlRepository {
    type Data = ProfileCreateData;
    type Output = Result<Profile, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let model = query::profile::insert(
            &self.db,
            Uuid::new_v4(),
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
        .await
        .map_err(|e| match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })?;

        Ok(model.into())
    }
}

#[async_trait::async_trait]
impl Updatable for ProfileSqlRepository {
    type Params = ProfileIdParams;

    type Data = ProfileUpdateData;

    type Output = Result<Profile, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, colette_core::Profile, Error>(|txn| {
                Box::pin(async move {
                    let Some(mut model) =
                        query::profile::select_by_id(txn, params.id, params.user_id)
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
}

#[async_trait::async_trait]
impl Deletable for ProfileSqlRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(profile) =
                        query::profile::select_by_id(txn, params.id, params.user_id)
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
}

#[async_trait::async_trait]
impl ProfileRepository for ProfileSqlRepository {
    async fn list(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<Profile>, Error> {
        find(&self.db, None, user_id, limit, cursor).await
    }

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<Uuid, Error>>, Error> {
        query::profile::stream(&self.db, feed_id)
            .await
            .map(|e| {
                e.map(|e| e.map_err(|e| Error::Unknown(e.into())))
                    .map_err(|e| Error::Unknown(e.into()))
                    .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<Profile>, Error> {
    let profiles = query::profile::select(db, id, user_id, limit, cursor)
        .await
        .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(profiles)
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    user_id: Uuid,
) -> Result<colette_core::Profile, Error> {
    let mut profiles = find(db, Some(id), user_id, None, None).await?;
    if profiles.is_empty() {
        return Err(Error::NotFound(id));
    }

    Ok(profiles.swap_remove(0))
}
