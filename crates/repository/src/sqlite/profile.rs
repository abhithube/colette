use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Error, ProfileCreateData, ProfileFindParams, ProfileIdParams, ProfileRepository,
        ProfileUpdateData,
    },
    Profile,
};
use deadpool_sqlite::{
    rusqlite::{self, Row},
    Pool,
};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteProfileRepository {
    pool: Pool,
}

impl SqliteProfileRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteProfileRepository {
    type Params = ProfileFindParams;
    type Output = Result<Vec<Profile>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::profile::select(
                params.id,
                params.user_id,
                params.is_default,
                params.cursor,
                params.limit,
            )
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut profiles: Vec<Profile> = Vec::new();
            while let Some(row) = rows.next()? {
                profiles.push(ProfileSelect::try_from(row).map(|e| e.0)?);
            }

            Ok(profiles)
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let title = data.title.clone();

        conn.interact(move |conn| {
            let (sql, values) = crate::profile::insert(
                Some(Uuid::new_v4()),
                data.title,
                data.image_url,
                None,
                data.user_id,
            )
            .build_rusqlite(SqliteQueryBuilder);

            conn.prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::Conflict(title)
            }
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
        if data.title.is_none() && data.image_url.is_none() {
            return Ok(());
        }

        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) =
                crate::profile::update(params.id, params.user_id, data.title, data.image_url)
                    .build_rusqlite(SqliteQueryBuilder);

            let count = conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
            if count == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteProfileRepository {
    type Params = ProfileIdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        {
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
        }

        conn.interact(move |conn| {
            let (sql, values) = crate::profile::delete(params.id, params.user_id)
                .build_rusqlite(SqliteQueryBuilder);

            conn.prepare_cached(&sql)?.execute(&*values.as_params())?;

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

impl ProfileRepository for SqliteProfileRepository {}

#[derive(Debug, Clone)]
pub(crate) struct ProfileSelect(Profile);

impl TryFrom<&Row<'_>> for ProfileSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(Profile {
            id: value.get("id")?,
            title: value.get("title")?,
            image_url: value.get("image_url")?,
            is_default: value.get("is_default")?,
            user_id: value.get("user_id")?,
        }))
    }
}
