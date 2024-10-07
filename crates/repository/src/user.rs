use colette_core::{
    common::{Creatable, Findable},
    user::{Error, NotFoundError, UserCreateData, UserIdParams, UserRepository},
    User,
};
use sqlx::{types::Uuid, PgPool};

pub struct UserSqlRepository {
    pub(crate) pool: PgPool,
}

impl UserSqlRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for UserSqlRepository {
    type Params = UserIdParams;
    type Output = Result<User, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        match params {
            UserIdParams::Id(id) => colette_postgres::user::select(&self.pool, Some(id), None)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Id(id)),
                    _ => Error::Unknown(e.into()),
                }),
            UserIdParams::Email(email) => {
                colette_postgres::user::select(&self.pool, None, Some(email.clone()))
                    .await
                    .map_err(|e| {
                        if let sqlx::Error::RowNotFound = e {
                            Error::NotFound(NotFoundError::Email(email))
                        } else {
                            Error::Unknown(e.into())
                        }
                    })
            }
        }
    }
}

#[async_trait::async_trait]
impl Creatable for UserSqlRepository {
    type Data = UserCreateData;
    type Output = Result<User, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let user_id = Uuid::new_v4();
        let user =
            colette_postgres::user::insert(&mut *tx, user_id, data.email.clone(), data.password)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.email)
                    }
                    _ => Error::Unknown(e.into()),
                })?;

        colette_postgres::profile::insert(
            &mut *tx,
            Uuid::new_v4(),
            "Default".to_owned(),
            None,
            Some(true),
            user_id,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(user)
    }
}

#[async_trait::async_trait]
impl UserRepository for UserSqlRepository {}
