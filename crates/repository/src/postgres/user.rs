use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserFindParams, UserRepository},
    User,
};
use deadpool_postgres::{
    tokio_postgres::{error::SqlState, Row},
    Pool,
};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: Pool,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        match params {
            UserFindParams::Id(id) => {
                let (sql, values) =
                    crate::user::select(Some(id), None).build_postgres(PostgresQueryBuilder);

                let stmt = client
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                if let Some(row) = client
                    .query_opt(&stmt, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                {
                    Ok(UserSelect::from(row).0)
                } else {
                    Err(Error::NotFound(NotFoundError::Id(id)))
                }
            }
            UserFindParams::Email(email) => {
                let (sql, values) = crate::user::select(None, Some(email.clone()))
                    .build_postgres(PostgresQueryBuilder);

                let stmt = client
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                if let Some(row) = client
                    .query_opt(&stmt, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                {
                    Ok(UserSelect::from(row).0)
                } else {
                    Err(Error::NotFound(NotFoundError::Email(email)))
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresUserRepository {
    type Data = UserCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = {
            let (sql, values) = crate::user::insert(None, data.email.clone(), data.password)
                .build_postgres(PostgresQueryBuilder);

            let stmt = client
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let row = client
                .query_one(&stmt, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.email),
                    _ => Error::Unknown(e.into()),
                })?;

            row.try_get::<_, Uuid>("id")
                .map_err(|e| Error::Unknown(e.into()))?
        };

        Ok(id)
    }
}

impl UserRepository for PostgresUserRepository {}

#[derive(Debug, Clone)]
struct UserSelect(User);

impl From<Row> for UserSelect {
    fn from(value: Row) -> Self {
        Self(User {
            id: value.get("id"),
            email: value.get("email"),
            password: value.get("password"),
        })
    }
}
