use colette_core::{
    user::{Error, NotFoundError, UserCreateData, UserFindOneParams, UserRepository},
    User,
};
use sea_orm::{SqlErr, TransactionError, TransactionTrait};
use uuid::Uuid;

use crate::{queries, SqlRepository};

#[async_trait::async_trait]
impl UserRepository for SqlRepository {
    async fn find_one_user(&self, params: UserFindOneParams) -> Result<User, Error> {
        match params {
            UserFindOneParams::Id(id) => {
                let Some(profile) = queries::user::select_by_id(&self.db, id)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(NotFoundError::Id(id)));
                };

                Ok(profile.into())
            }
            UserFindOneParams::Email(email) => {
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

    async fn create_user(&self, data: UserCreateData) -> Result<User, Error> {
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
