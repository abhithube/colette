use colette_core::{
    user::{Error, NotFoundError, UserCreateData, UserFindOneParams, UserRepository},
    User,
};
use colette_entities::{profile, user};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, Set, SqlErr, TransactionError, TransactionTrait,
};
use uuid::Uuid;

use crate::SqlRepository;

#[async_trait::async_trait]
impl UserRepository for SqlRepository {
    async fn find_one_user(&self, params: UserFindOneParams) -> Result<User, Error> {
        match params {
            UserFindOneParams::Id(id) => {
                let Some(profile) = user::Entity::find_by_id(id)
                    .one(&self.db)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
                else {
                    return Err(Error::NotFound(NotFoundError::Id(id)));
                };

                Ok(profile.into())
            }
            UserFindOneParams::Email(email) => {
                let Some(profile) = user::Entity::find()
                    .filter(user::Column::Email.eq(email.clone()))
                    .one(&self.db)
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
                    let user_model = user::ActiveModel {
                        id: Set(user_id),
                        email: Set(data.email.clone()),
                        password: Set(data.password),
                        ..Default::default()
                    };

                    let user = user::Entity::insert(user_model)
                        .exec_with_returning(txn)
                        .await
                        .map_err(|e| match e.sql_err() {
                            Some(SqlErr::UniqueConstraintViolation(_)) => {
                                Error::Conflict(data.email)
                            }
                            _ => Error::Unknown(e.into()),
                        })?;

                    let profile_model = profile::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        title: Set("Default".to_owned()),
                        is_default: Set(true),
                        user_id: Set(user_id),
                        ..Default::default()
                    };

                    profile::Entity::insert(profile_model)
                        .exec_without_returning(txn)
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    Ok(user.into())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
    }
}
