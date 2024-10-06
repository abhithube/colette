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
use sea_orm::{prelude::Uuid, sqlx, DatabaseConnection};

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
impl Creatable for ProfileSqlRepository {
    type Data = ProfileCreateData;
    type Output = Result<Profile, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        colette_postgres::profile::insert(
            self.db.get_postgres_connection_pool(),
            Uuid::new_v4(),
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for ProfileSqlRepository {
    type Params = ProfileIdParams;
    type Data = ProfileUpdateData;
    type Output = Result<Profile, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        colette_postgres::profile::update(
            self.db.get_postgres_connection_pool(),
            params.id,
            params.user_id,
            data.title,
            Some(data.image_url),
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Deletable for ProfileSqlRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let profiles = colette_postgres::profile::select(
            &mut *tx,
            Some(params.id),
            params.user_id,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(profile) = profiles.first() {
            if profile.is_default {
                return Err(Error::DeletingDefault);
            }
        } else {
            return Err(Error::NotFound(params.id));
        }

        colette_postgres::profile::delete(&mut *tx, params.id, params.user_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
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
        find(&self.db, None, user_id, None, limit, cursor).await
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

async fn find(
    db: &DatabaseConnection,
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<Profile>, Error> {
    let profiles = colette_postgres::profile::select(
        db.get_postgres_connection_pool(),
        id,
        user_id,
        is_default,
        cursor,
        limit,
    )
    .await
    .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
    .map_err(|e| Error::Unknown(e.into()))?;

    Ok(profiles)
}
