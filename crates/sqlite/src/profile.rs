use anyhow::anyhow;
use colette_core::{
    common::{Creatable, Deletable, Findable, Updatable},
    profile::{
        Cursor, Error, ProfileCreateData, ProfileIdOrDefaultParams, ProfileIdParams,
        ProfileRepository, ProfileUpdateData,
    },
    Profile,
};
use deadpool_sqlite::Pool;
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};
use rusqlite::{Connection, Row};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SqliteProfileRepository {
    pub(crate) pool: Pool,
}

impl SqliteProfileRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteProfileRepository {
    type Params = ProfileIdOrDefaultParams;
    type Output = Result<Profile, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        match params.id {
            Some(id) => conn
                .interact(move |conn| find_by_id(conn, id, params.user_id))
                .await
                .unwrap()
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Error::NotFound(id),
                    _ => Error::Unknown(e.into()),
                }),
            None => {
                let mut profiles = conn
                    .interact(move |conn| find(conn, None, params.user_id, Some(true), None, None))
                    .await
                    .unwrap()
                    .map_err(|e| Error::Unknown(e.into()))?;

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
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteProfileRepository {
    type Data = ProfileCreateData;
    type Output = Result<Profile, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let title = data.title.clone();

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let id = Uuid::new_v4();

            {
                let (sql, values) = colette_sql::profile::insert(
                    id,
                    data.title,
                    data.image_url,
                    None,
                    data.user_id,
                )
                .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
            }

            let profile = find_by_id(&tx, id, data.user_id)?;

            tx.commit()?;

            Ok(profile)
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
    type Output = Result<Profile, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            if data.title.is_some() || data.image_url.is_some() {
                let (sql, values) = colette_sql::profile::update(
                    params.id,
                    params.user_id,
                    data.title,
                    data.image_url,
                )
                .build_rusqlite(SqliteQueryBuilder);

                let count = tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
                if count == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
            }

            let profile = find_by_id(&tx, params.id, params.user_id)?;

            tx.commit()?;

            Ok(profile)
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

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let profile = find_by_id(&tx, params.id, params.user_id)?;
            if profile.is_default {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error {
                        code: rusqlite::ErrorCode::ConstraintViolation,
                        extended_code: rusqlite::ffi::SQLITE_CONSTRAINT,
                    },
                    None,
                ));
            }

            let (sql, values) = colette_sql::profile::delete(params.id, params.user_id)
                .build_rusqlite(SqliteQueryBuilder);

            tx.prepare_cached(&sql)?.execute(&*values.as_params())?;

            tx.commit()?;

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::DeletingDefault
            }
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl ProfileRepository for SqliteProfileRepository {
    async fn list(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<Profile>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find(conn, None, user_id, None, limit, cursor))
            .await
            .unwrap()
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn stream(&self, feed_id: i32) -> Result<BoxStream<Result<Uuid, Error>>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT DISTINCT profile_id AS id FROM profile_feed WHERE feed_id = ?",
            )?;
            let rows = stmt.query_map([feed_id], |row| row.get::<_, Uuid>("id"))?;

            Ok::<_, rusqlite::Error>(rows.into_iter().collect::<Vec<_>>())
        })
        .await
        .unwrap()
        .map(|e| {
            stream::iter(
                e.into_iter()
                    .map(|e| e.map_err(|e| Error::Unknown(e.into())))
                    .collect::<Vec<_>>(),
            )
            .boxed()
        })
        .map_err(|e| Error::Unknown(e.into()))
    }
}

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

fn find(
    conn: &Connection,
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> rusqlite::Result<Vec<Profile>> {
    let (sql, values) = colette_sql::profile::select(id, user_id, is_default, cursor, limit)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(&*values.as_params())?;

    let mut profiles: Vec<Profile> = Vec::new();
    while let Some(row) = rows.next()? {
        profiles.push(ProfileSelect::try_from(row).map(|e| e.0)?);
    }

    Ok(profiles)
}

fn find_by_id(conn: &Connection, id: Uuid, user_id: Uuid) -> rusqlite::Result<Profile> {
    let mut profiles = find(conn, Some(id), user_id, None, None, None)?;
    if profiles.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(profiles.swap_remove(0))
}
