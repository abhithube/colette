use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Cursor, Error, ProfileCreateData, ProfileIdOrDefaultParams, ProfileIdParams,
        ProfileRepository, ProfileUpdateData,
    },
    Profile,
};
use deadpool_postgres::{GenericClient, Pool};
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{error::SqlState, Row};
use uuid::Uuid;

pub struct PostgresProfileRepository {
    pub(crate) pool: Pool,
}

impl PostgresProfileRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresProfileRepository {
    type Params = ProfileIdOrDefaultParams;
    type Output = Result<Profile, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let is_default = params.id.map_or_else(|| Some(true), |_| None);

        let mut profiles = find(&client, params.id, params.user_id, is_default, None, None).await?;
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
impl Creatable for PostgresProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Profile, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = Uuid::new_v4();

        let (sql, values) = colette_sql::profile::insert(
            id,
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
        .build_postgres(PostgresQueryBuilder);

        tx.execute(&sql, &values.as_params())
            .await
            .map_err(|e| match e.code() {
                Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        let profile = find_by_id(
            &tx,
            ProfileIdParams {
                id,
                user_id: data.user_id,
            },
        )
        .await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(profile)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresProfileRepository {
    type Params = ProfileIdParams;
    type Data = ProfileUpdateData;
    type Output = Result<Profile, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() || data.image_url.is_some() {
            let count = {
                let (sql, values) = colette_sql::profile::update(
                    params.id,
                    params.user_id,
                    data.title,
                    data.image_url,
                )
                .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        let profile = find_by_id(&tx, params)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(profile)
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresProfileRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let profile = find_by_id(&tx, params.clone()).await?;
        if profile.is_default {
            return Err(Error::DeletingDefault);
        }

        {
            let (sql, values) = colette_sql::profile::delete(params.id, params.user_id)
                .build_postgres(PostgresQueryBuilder);

            tx.execute(&sql, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl ProfileRepository for PostgresProfileRepository {
    async fn list(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<Profile>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find(&client, None, user_id, None, limit, cursor).await
    }

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<Uuid, Error>>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .query_raw(
                "SELECT DISTINCT profile_id FROM profile_feed WHERE feed_id = $1",
                &[&feed_id],
            )
            .await
            .map(|e| {
                e.map(|e| e.map(|e| e.get("id")))
                    .map_err(|e| Error::Unknown(e.into()))
                    .boxed()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProfileSelect(Profile);

impl From<&Row> for ProfileSelect {
    fn from(value: &Row) -> Self {
        Self(Profile {
            id: value.get("id"),
            title: value.get("title"),
            image_url: value.get("image_url"),
            is_default: value.get("is_default"),
            user_id: value.get("user_id"),
        })
    }
}

async fn find<C: GenericClient>(
    client: &C,
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<Profile>, Error> {
    let (sql, values) = colette_sql::profile::select(id, user_id, is_default, cursor, limit)
        .build_postgres(PostgresQueryBuilder);

    client
        .query(&sql, &values.as_params())
        .await
        .map(|e| {
            e.into_iter()
                .map(|e| ProfileSelect::from(&e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id<C: GenericClient>(
    client: &C,
    params: ProfileIdParams,
) -> Result<Profile, Error> {
    let mut profiles = find(client, Some(params.id), params.user_id, None, None, None).await?;
    if profiles.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(profiles.swap_remove(0))
}
