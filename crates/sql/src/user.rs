use colette_core::{
    common::Creatable,
    user::{Error, NotFoundError, UserCreateData, UserIdParams, UserRepository},
    User,
};
use sea_orm::{DatabaseConnection, SqlErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::queries;

pub struct UserSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl UserSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Creatable for UserSqlRepository {
    type Data = UserCreateData;
    type Output = Result<User, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, User, Error>(|txn| {
                Box::pin(async move {
                    let user_id = Uuid::new_v4();
                    let model =
                        queries::user::insert(txn, user_id, data.email.clone(), data.password)
                            .await
                            .map_err(|e| match e.sql_err() {
                                Some(SqlErr::UniqueConstraintViolation(_)) => {
                                    Error::Conflict(data.email)
                                }
                                _ => Error::Unknown(e.into()),
                            })?;

                    queries::profile::insert(
                        txn,
                        Uuid::new_v4(),
                        "Default".to_owned(),
                        None,
                        Some(true),
                        user_id,
                    )
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(model.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl UserRepository for UserSqlRepository {
    async fn find_one(&self, params: UserIdParams) -> Result<User, Error> {
        match params {
            UserIdParams::Id(id) => {
                let Some(profile) = queries::user::select_by_id(&self.db, id)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(NotFoundError::Id(id)));
                };

                Ok(profile.into())
            }
            UserIdParams::Email(email) => {
                let Some(profile) = queries::user::select_by_email(&self.db, email.clone())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(NotFoundError::Email(email)));
                };

                Ok(profile.into())
            }
        }
    }
}
