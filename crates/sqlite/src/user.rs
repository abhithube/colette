use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserFindParams, UserRepository},
    User,
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteUserRepository {
    type Params = UserFindParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params {
            UserFindParams::Id(id) => {
                let (sql, values) =
                    colette_sql::user::select(Some(id), None).build_sqlx(SqliteQueryBuilder);

                sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
                    .fetch_one(&self.pool)
                    .await
                    .map(User::from)
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Id(id)),
                        _ => Error::Unknown(e.into()),
                    })
            }
            UserFindParams::Email(email) => {
                let (sql, values) = colette_sql::user::select(None, Some(email.clone()))
                    .build_sqlx(SqliteQueryBuilder);

                sqlx::query_as_with::<_, UserSelect, _>(&sql, values)
                    .fetch_one(&self.pool)
                    .await
                    .map(User::from)
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Email(email)),
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = {
            let (sql, values) =
                colette_sql::user::insert(Some(Uuid::new_v4()), data.email.clone(), data.password)
                    .build_sqlx(SqliteQueryBuilder);

            sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.email)
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        {
            let (sql, values) = colette_sql::profile::insert(
                Some(Uuid::new_v4()),
                "Default".to_owned(),
                None,
                Some(true),
                id,
            )
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(id)
    }
}

impl UserRepository for SqliteUserRepository {}

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
