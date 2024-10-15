use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserIdParams, UserRepository},
    User,
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{error::SqlState, Row};
use uuid::Uuid;

pub struct PostgresUserRepository {
    pub(crate) pool: Pool,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresUserRepository {
    type Params = UserIdParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        match params {
            UserIdParams::Id(id) => {
                let (sql, values) =
                    colette_sql::user::select(Some(id), None).build_postgres(PostgresQueryBuilder);

                let stmt = client
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                if let Some(row) = client
                    .query_opt(&stmt, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                {
                    Ok(UserSelect::from(&row).0)
                } else {
                    Err(Error::NotFound(NotFoundError::Id(id)))
                }
            }
            UserIdParams::Email(email) => {
                let (sql, values) = colette_sql::user::select(None, Some(email.clone()))
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
                    Ok(UserSelect::from(&row).0)
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
    type Output = Result<User, Error>;

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

        let user = {
            let (sql, values) =
                colette_sql::user::insert(Uuid::new_v4(), data.email.clone(), data.password)
                    .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let row = tx
                .query_one(&stmt, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.email),
                    _ => Error::Unknown(e.into()),
                })?;

            UserSelect::from(&row).0
        };

        {
            let (sql, values) = colette_sql::profile::insert(
                Uuid::new_v4(),
                "Default".to_owned(),
                None,
                Some(true),
                user.id,
            )
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(user)
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {}

#[derive(Debug, Clone)]
struct UserSelect(colette_core::User);

impl From<&Row> for UserSelect {
    fn from(value: &Row) -> Self {
        Self(User {
            id: value.get("id"),
            email: value.get("email"),
            password: value.get("password"),
        })
    }
}
