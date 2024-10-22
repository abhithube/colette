use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserIdParams, UserRepository},
    User,
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresUserRepository {
    type Params = UserIdParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params {
            UserIdParams::Id(id) => {
                let (sql, values) =
                    colette_sql::user::select(Some(id), None).build_sqlx(PostgresQueryBuilder);

                sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
                    .fetch_one(&self.pool)
                    .await
                    .map(|e| e.into())
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Id(id)),
                        _ => Error::Unknown(e.into()),
                    })
            }
            UserIdParams::Email(email) => {
                let (sql, values) = colette_sql::user::select(None, Some(email.clone()))
                    .build_sqlx(PostgresQueryBuilder);

                sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
                    .fetch_one(&self.pool)
                    .await
                    .map(|e| e.into())
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Email(email)),
                        _ => Error::Unknown(e.into()),
                    })
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresUserRepository {
    type Data = UserCreateData;
    type Output = Result<User, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let user = {
            let (sql, values) = colette_sql::user::insert(None, data.email.clone(), data.password)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map(User::from)
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.email)
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        {
            let (sql, values) =
                colette_sql::profile::insert(None, "Default".to_owned(), None, Some(true), user.id)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(user)
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {}

#[derive(Debug, Clone, sqlx::FromRow)]
struct UserSelect {
    id: Uuid,
    email: String,
    password: String,
}

impl From<UserSelect> for colette_core::User {
    fn from(value: UserSelect) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
        }
    }
}
