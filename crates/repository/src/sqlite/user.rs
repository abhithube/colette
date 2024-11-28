use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserFindParams, UserRepository},
    User,
};
use deadpool_sqlite::{
    rusqlite::{self, Row},
    Pool,
};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    pool: Pool,
}

impl SqliteUserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        match params {
            UserFindParams::Id(id) => {
                let (sql, values) =
                    crate::user::select(Some(id), None).build_rusqlite(SqliteQueryBuilder);

                conn.interact(move |conn| {
                    conn.prepare_cached(&sql)?
                        .query_row(&*values.as_params(), |row| {
                            UserSelect::try_from(row).map(|e| e.0)
                        })
                })
                .await
                .unwrap()
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Error::NotFound(NotFoundError::Id(id)),
                    _ => Error::Unknown(e.into()),
                })
            }
            UserFindParams::Email(email) => {
                let (sql, values) = crate::user::select(None, Some(email.clone()))
                    .build_rusqlite(SqliteQueryBuilder);

                conn.interact(move |conn| {
                    conn.prepare_cached(&sql)?
                        .query_row(&*values.as_params(), |row| {
                            UserSelect::try_from(row).map(|e| e.0)
                        })
                })
                .await
                .unwrap()
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        Error::NotFound(NotFoundError::Email(email))
                    }
                    _ => Error::Unknown(e.into()),
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteUserRepository {
    type Data = UserCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let email = data.email.clone();

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let (sql, values) =
                crate::user::insert(Some(Uuid::new_v4()), data.email, data.password)
                    .build_rusqlite(SqliteQueryBuilder);

            let id = tx
                .prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?;

            {
                let (sql, values) = crate::profile::insert(
                    Some(Uuid::new_v4()),
                    "Default".to_owned(),
                    None,
                    Some(true),
                    id,
                )
                .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?
                    .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?;
            }

            tx.commit()?;

            Ok(id)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::Conflict(email)
            }
            _ => Error::Unknown(e.into()),
        })
    }
}

impl UserRepository for SqliteUserRepository {}

#[derive(Debug, Clone)]
struct UserSelect(User);

impl TryFrom<&Row<'_>> for UserSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(User {
            id: value.get("id")?,
            email: value.get("email")?,
            password: value.get("password")?,
        }))
    }
}
