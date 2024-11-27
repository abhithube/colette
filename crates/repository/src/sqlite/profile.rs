use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Error, ProfileCreateData, ProfileFindParams, ProfileIdParams, ProfileRepository,
        ProfileUpdateData,
    },
    Profile,
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteProfileRepository {
    pool: SqlitePool,
}

impl SqliteProfileRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteProfileRepository {
    type Params = ProfileFindParams;
    type Output = Result<Vec<Profile>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::profile::select(
            params.id,
            params.user_id,
            params.is_default,
            params.cursor,
            params.limit,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(Profile::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let (sql, values) = crate::profile::insert(
            Some(Uuid::new_v4()),
            data.title.clone(),
            data.image_url,
            None,
            data.user_id,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteProfileRepository {
    type Params = ProfileIdParams;
    type Data = ProfileUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() || data.image_url.is_some() {
            let count = {
                let (sql, values) = crate::profile::update(
                    params.id,
                    params.user_id,
                    data.title,
                    data.image_url,
                )
                .build_sqlx(SqliteQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&self.pool)
                    .await
                    .map(|e| e.rows_affected())
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteProfileRepository {
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

        let (sql, values) =
            crate::profile::delete(params.id, params.user_id).build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

impl ProfileRepository for SqliteProfileRepository {}

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
