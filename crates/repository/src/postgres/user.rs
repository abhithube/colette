use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserFindParams, UserRepository},
    User,
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: Pool<Postgres>,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params {
            UserFindParams::Id(id) => {
                let (sql, values) =
                    crate::user::select(Some(id), None).build_sqlx(PostgresQueryBuilder);

                if let Some(row) = sqlx::query_with(&sql, values)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                {
                    Ok(UserSelect::from(row).0)
                } else {
                    Err(Error::NotFound(NotFoundError::Id(id)))
                }
            }
            UserFindParams::Email(email) => {
                let (sql, values) =
                    crate::user::select(None, Some(email.clone())).build_sqlx(PostgresQueryBuilder);

                if let Some(row) = sqlx::query_with(&sql, values)
                    .fetch_optional(&self.pool)
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
        let id = {
            let (sql, values) = crate::user::insert(None, data.email.clone(), data.password)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.email)
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        Ok(id)
    }
}

impl UserRepository for PostgresUserRepository {}

#[derive(Debug, Clone)]
struct UserSelect(User);

impl From<PgRow> for UserSelect {
    fn from(value: PgRow) -> Self {
        Self(User {
            id: value.get("id"),
            email: value.get("email"),
            password: value.get("password"),
        })
    }
}
