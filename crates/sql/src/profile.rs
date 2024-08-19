use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Paginated},
    profile::{Error, ProfileCreateData, ProfileRepository, ProfileUpdateData, StreamProfile},
    Profile,
};
use colette_utils::base_64;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, ModelTrait, SqlErr,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::queries;

pub struct ProfileSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl ProfileSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Creatable for ProfileSqlRepository {
    type Data = ProfileCreateData;
    type Output = Result<Profile, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let model = queries::profile::insert(
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
impl ProfileRepository for ProfileSqlRepository {
    async fn find_many(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
    ) -> Result<Paginated<Profile>, Error> {
        find(&self.db, None, user_id, limit, cursor_raw).await
    }

    async fn find_one(&self, id: Option<Uuid>, user_id: Uuid) -> Result<Profile, Error> {
        match id {
            Some(id) => find_by_id(&self.db, id, user_id).await,
            None => {
                let Some(profile) = queries::profile::select_default(&self.db, user_id)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::Unknown(anyhow!("couldn't find default profile")));
                };

                Ok(profile.into())
            }
        }
    }

    async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        data: ProfileUpdateData,
    ) -> Result<Profile, Error> {
        self.db
            .transaction::<_, colette_core::Profile, Error>(|txn| {
                Box::pin(async move {
                    let Some(mut model) = queries::profile::select_by_id(txn, id, user_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
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

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(profile) = queries::profile::select_by_id(txn, id, user_id)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(id));
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

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<StreamProfile, Error>>, Error> {
        queries::profile::stream(&self.db, feed_id)
            .await
            .map(|e| {
                e.map(|e| {
                    e.map(StreamProfile::from)
                        .map_err(|e| Error::Unknown(e.into()))
                })
                .map_err(|e| Error::Unknown(e.into()))
                .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

impl From<queries::profile::StreamSelect> for StreamProfile {
    fn from(value: queries::profile::StreamSelect) -> Self {
        Self { id: value.id }
    }
}

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
) -> Result<Paginated<Profile>, Error> {
    let mut cursor = Cursor::default();
    if let Some(raw) = cursor_raw.as_deref() {
        cursor = base_64::decode::<Cursor>(raw)?;
    }

    let mut profiles = queries::profile::select(db, id, user_id, limit.map(|e| e + 1), cursor)
        .await
        .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if profiles.len() > limit {
            profiles = profiles.into_iter().take(limit).collect();

            if let Some(last) = profiles.last() {
                let c = Cursor {
                    title: last.title.to_owned(),
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<Profile> {
        cursor,
        data: profiles,
    })
}

async fn find_by_id<Db: ConnectionTrait>(
    db: &Db,
    id: Uuid,
    user_id: Uuid,
) -> Result<colette_core::Profile, Error> {
    let profiles = find(db, Some(id), user_id, Some(1), None).await?;

    profiles.data.first().cloned().ok_or(Error::NotFound(id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}
