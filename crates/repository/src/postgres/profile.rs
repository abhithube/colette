use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Error, ProfileCreateData, ProfileFindParams, ProfileIdParams, ProfileRepository,
        ProfileUpdateData,
    },
    Profile,
};
use deadpool_postgres::{
    tokio_postgres::{error::SqlState, Row},
    Pool,
};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresProfileRepository {
    pool: Pool,
}

impl PostgresProfileRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresProfileRepository {
    type Params = ProfileFindParams;
    type Output = Result<Vec<Profile>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::profile::select(
            params.id,
            params.user_id,
            params.is_default,
            params.cursor,
            params.limit,
        )
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .query(&stmt, &values.as_params())
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| ProfileSelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) =
            crate::profile::insert(None, data.title.clone(), data.image_url, None, data.user_id)
                .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = client
            .query_one(&stmt, &values.as_params())
            .await
            .map(|e| e.get::<_, Uuid>("id"))
            .map_err(|e| match e.code() {
                Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresProfileRepository {
    type Params = ProfileIdParams;
    type Data = ProfileUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() || data.image_url.is_some() {
            let (sql, values) =
                crate::profile::update(params.id, params.user_id, data.title, data.image_url)
                    .build_postgres(PostgresQueryBuilder);

            let stmt = client
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let count = client
                .execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresProfileRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let mut profiles = self
            .find(ProfileFindParams {
                id: Some(params.id),
                user_id: params.user_id,
                ..Default::default()
            })
            .await?;
        if profiles.is_empty() {
            return Err(Error::NotFound(params.id));
        }

        let profile = profiles.swap_remove(0);
        if profile.is_default {
            return Err(Error::DeletingDefault);
        }

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) =
            crate::profile::delete(params.id, params.user_id).build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

impl ProfileRepository for PostgresProfileRepository {}

#[derive(Debug, Clone)]
pub(crate) struct ProfileSelect(Profile);

impl From<Row> for ProfileSelect {
    fn from(value: Row) -> Self {
        Self(Profile {
            id: value.get("id"),
            title: value.get("title"),
            image_url: value.get("image_url"),
            is_default: value.get("is_default"),
            user_id: value.get("user_id"),
        })
    }
}
