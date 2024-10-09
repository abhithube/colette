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
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor, PgPool};

pub struct PostgresProfileRepository {
    pub(crate) pool: PgPool,
}

impl PostgresProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresProfileRepository {
    type Params = ProfileIdOrDefaultParams;
    type Output = Result<Profile, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let is_default = params.id.map_or_else(|| Some(true), |_| None);

        let mut profiles = find(
            &self.pool,
            params.id,
            params.user_id,
            is_default,
            None,
            None,
        )
        .await?;
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
        let (sql, values) = colette_sql::profile::insert(
            Uuid::new_v4(),
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map(|e| e.into())
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresProfileRepository {
    type Params = ProfileIdParams;
    type Data = ProfileUpdateData;
    type Output = Result<Profile, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() || data.image_url.is_some() {
            let result = {
                let (sql, values) = colette_sql::profile::update(
                    params.id,
                    params.user_id,
                    data.title,
                    data.image_url,
                )
                .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if result.rows_affected() == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        let mut profiles =
            find(&mut *tx, Some(params.id), params.user_id, None, None, None).await?;
        if profiles.is_empty() {
            return Err(Error::NotFound(params.id));
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(profiles.swap_remove(0))
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresProfileRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let profiles = {
            let (sql, values) =
                colette_sql::profile::select(Some(params.id), params.user_id, None, None, None)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await
                .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
                .map_err(|e| Error::Unknown(e.into()))?
        };

        if let Some(profile) = profiles.first() {
            if profile.is_default {
                return Err(Error::DeletingDefault);
            }
        } else {
            return Err(Error::NotFound(params.id));
        }

        {
            let (sql, values) = colette_sql::profile::delete(params.id, params.user_id)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
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
        find(&self.pool, None, user_id, None, limit, cursor).await
    }

    fn stream(&self, feed_id: i32) -> BoxStream<Result<Uuid, Error>> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT DISTINCT profile_id FROM profile_feed WHERE feed_id = $1",
        )
        .bind(feed_id)
        .fetch(&self.pool)
        .map_err(|e| Error::Unknown(e.into()))
        .boxed()
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct ProfileSelect {
    id: Uuid,
    title: String,
    image_url: Option<String>,
    is_default: bool,
    user_id: Uuid,
}

impl From<ProfileSelect> for colette_core::Profile {
    fn from(value: ProfileSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            is_default: value.is_default,
            user_id: value.user_id,
        }
    }
}

async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> Result<Vec<Profile>, Error> {
    let (sql, values) = colette_sql::profile::select(id, user_id, is_default, cursor, limit)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))
}
