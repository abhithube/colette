use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserIdParams, UserRepository},
    User,
};
use deadpool_sqlite::Pool;
use rusqlite::Row;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SqliteUserRepository {
    pub(crate) pool: Pool,
}

impl SqliteUserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteUserRepository {
    type Params = UserIdParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        match params {
            UserIdParams::Id(id) => {
                let (sql, values) =
                    colette_sql::user::select(Some(id), None).build_rusqlite(SqliteQueryBuilder);

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
            UserIdParams::Email(email) => {
                let (sql, values) = colette_sql::user::select(None, Some(email.clone()))
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
    type Output = Result<User, Error>;

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
                colette_sql::user::insert(Uuid::new_v4(), data.email, data.password)
                    .build_rusqlite(SqliteQueryBuilder);

            let user = tx
                .prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| {
                    UserSelect::try_from(row).map(|e| e.0)
                })?;

            {
                let (sql, values) = colette_sql::profile::insert(
                    Uuid::new_v4(),
                    "Default".to_owned(),
                    None,
                    Some(true),
                    user.id,
                )
                .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
            }

            tx.commit()?;

            Ok(user)
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

#[async_trait::async_trait]
impl UserRepository for SqliteUserRepository {}

#[derive(Debug, Clone)]
struct UserSelect(colette_core::User);

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
